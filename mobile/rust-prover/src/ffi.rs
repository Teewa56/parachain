use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::proving::{self, ProofResult};

#[derive(Deserialize)]
struct ProverRequest {
    circuit_id: String,
    public_inputs: PublicInputs,
    private_inputs_b64: String,
    proving_key_b64: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PublicInputs {
    Age {
        current_timestamp: u64,
        age_threshold_years: u64,
        credential_type_hash_b64: String,
    },
    Student {
        current_timestamp: u64,
        institution_hash_b64: String,
        status_active: bool,
    },
    Vaccination {
        current_timestamp: u64,
        vaccination_type_hash_b64: String,
        min_doses_required: u8,
    },
    Employment {
        current_timestamp: u64,
        company_hash_b64: String,
        employment_type_hash_b64: String,
    },
}

#[derive(Deserialize)]
struct AgePrivateInputs {
    birth_timestamp: u64,
    credential_hash_b64: String,
    issuer_signature_hash_b64: String,
}

#[derive(Deserialize)]
struct StudentPrivateInputs {
    student_id_hash_b64: String,
    enrollment_date: u64,
    expiry_date: u64,
    gpa: u16,
    credential_hash_b64: String,
    issuer_signature_hash_b64: String,
}

#[derive(Deserialize)]
struct VaccinationPrivateInputs {
    patient_id_hash_b64: String,
    vaccination_date: u64,
    expiry_date: u64,
    doses_received: u8,
    batch_number_hash_b64: String,
    credential_hash_b64: String,
    issuer_signature_hash_b64: String,
}

#[derive(Deserialize)]
struct EmploymentPrivateInputs {
    employee_id_hash_b64: String,
    start_date: u64,
    end_date: u64,
    salary: u64,
    position_hash_b64: String,
    credential_hash_b64: String,
    issuer_signature_hash_b64: String,
}

#[derive(Serialize)]
struct ProverResponse {
    ok: bool,
    proof_base64: Option<String>,
    public_inputs: Option<Vec<String>>,
    error: Option<String>,
}

#[no_mangle]
pub extern "C" fn generate_proof_json(request_ptr: *const c_char) -> *mut c_char {
    // Safety checks
    if request_ptr.is_null() {
        return error_response("null request pointer");
    }

    // Parse request
    let c_str = unsafe { CStr::from_ptr(request_ptr) };
    let req_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return error_response("invalid utf8"),
    };

    let req: ProverRequest = match serde_json::from_str(req_str) {
        Ok(r) => r,
        Err(e) => return error_response(&format!("json parse error: {}", e)),
    };

    // Decode proving key
    let proving_key_bytes = match base64::decode(&req.proving_key_b64) {
        Ok(b) => b,
        Err(e) => return error_response(&format!("proving key decode error: {}", e)),
    };

    // Route to appropriate circuit
    let result = match req.circuit_id.as_str() {
        "age_verification" => generate_age_proof_wrapper(&req.public_inputs, &req.private_inputs_b64, &proving_key_bytes),
        "student_status" => generate_student_proof_wrapper(&req.public_inputs, &req.private_inputs_b64, &proving_key_bytes),
        "vaccination_status" => generate_vaccination_proof_wrapper(&req.public_inputs, &req.private_inputs_b64, &proving_key_bytes),
        "employment_status" => generate_employment_proof_wrapper(&req.public_inputs, &req.private_inputs_b64, &proving_key_bytes),
        _ => Err(format!("unknown circuit: {}", req.circuit_id).into()),
    };

    // Build response
    match result {
        Ok(proof_result) => success_response(proof_result),
        Err(e) => error_response(&e.to_string()),
    }
}

fn generate_age_proof_wrapper(
    public_inputs: &PublicInputs,
    private_inputs_b64: &str,
    proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn std::error::Error>> {
    let (current_timestamp, age_threshold_years, credential_type_hash_b64) = match public_inputs {
        PublicInputs::Age { current_timestamp, age_threshold_years, credential_type_hash_b64 } => {
            (*current_timestamp, *age_threshold_years, credential_type_hash_b64.as_str())
        }
        _ => return Err("invalid public inputs for age_verification".into()),
    };

    let credential_type_hash = decode_hash32(credential_type_hash_b64)?;
    let private: AgePrivateInputs = serde_json::from_str(private_inputs_b64)?;
    
    let credential_hash = decode_hash32(&private.credential_hash_b64)?;
    let issuer_signature_hash = decode_hash32(&private.issuer_signature_hash_b64)?;

    proving::generate_age_proof(
        current_timestamp,
        age_threshold_years,
        &credential_type_hash,
        private.birth_timestamp,
        &credential_hash,
        &issuer_signature_hash,
        proving_key_bytes,
    )
}

fn generate_student_proof_wrapper(
    public_inputs: &PublicInputs,
    private_inputs_b64: &str,
    proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn std::error::Error>> {
    let (current_timestamp, institution_hash_b64, status_active) = match public_inputs {
        PublicInputs::Student { current_timestamp, institution_hash_b64, status_active } => {
            (*current_timestamp, institution_hash_b64.as_str(), *status_active)
        }
        _ => return Err("invalid public inputs for student_status".into()),
    };

    let institution_hash = decode_hash32(institution_hash_b64)?;
    let private: StudentPrivateInputs = serde_json::from_str(private_inputs_b64)?;
    
    let student_id_hash = decode_hash32(&private.student_id_hash_b64)?;
    let credential_hash = decode_hash32(&private.credential_hash_b64)?;
    let issuer_signature_hash = decode_hash32(&private.issuer_signature_hash_b64)?;

    proving::generate_student_proof(
        current_timestamp,
        &institution_hash,
        status_active,
        &student_id_hash,
        private.enrollment_date,
        private.expiry_date,
        private.gpa,
        &credential_hash,
        &issuer_signature_hash,
        proving_key_bytes,
    )
}

fn generate_vaccination_proof_wrapper(
    public_inputs: &PublicInputs,
    private_inputs_b64: &str,
    proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn std::error::Error>> {
    let (current_timestamp, vaccination_type_hash_b64, min_doses_required) = match public_inputs {
        PublicInputs::Vaccination { current_timestamp, vaccination_type_hash_b64, min_doses_required } => {
            (*current_timestamp, vaccination_type_hash_b64.as_str(), *min_doses_required)
        }
        _ => return Err("invalid public inputs for vaccination_status".into()),
    };

    let vaccination_type_hash = decode_hash32(vaccination_type_hash_b64)?;
    let private: VaccinationPrivateInputs = serde_json::from_str(private_inputs_b64)?;
    
    let patient_id_hash = decode_hash32(&private.patient_id_hash_b64)?;
    let batch_number_hash = decode_hash32(&private.batch_number_hash_b64)?;
    let credential_hash = decode_hash32(&private.credential_hash_b64)?;
    let issuer_signature_hash = decode_hash32(&private.issuer_signature_hash_b64)?;

    proving::generate_vaccination_proof(
        current_timestamp,
        &vaccination_type_hash,
        min_doses_required,
        &patient_id_hash,
        private.vaccination_date,
        private.expiry_date,
        private.doses_received,
        &batch_number_hash,
        &credential_hash,
        &issuer_signature_hash,
        proving_key_bytes,
    )
}

fn generate_employment_proof_wrapper(
    public_inputs: &PublicInputs,
    private_inputs_b64: &str,
    proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn std::error::Error>> {
    let (current_timestamp, company_hash_b64, employment_type_hash_b64) = match public_inputs {
        PublicInputs::Employment { current_timestamp, company_hash_b64, employment_type_hash_b64 } => {
            (*current_timestamp, company_hash_b64.as_str(), employment_type_hash_b64.as_str())
        }
        _ => return Err("invalid public inputs for employment_status".into()),
    };

    let company_hash = decode_hash32(company_hash_b64)?;
    let employment_type_hash = decode_hash32(employment_type_hash_b64)?;
    let private: EmploymentPrivateInputs = serde_json::from_str(private_inputs_b64)?;
    
    let employee_id_hash = decode_hash32(&private.employee_id_hash_b64)?;
    let position_hash = decode_hash32(&private.position_hash_b64)?;
    let credential_hash = decode_hash32(&private.credential_hash_b64)?;
    let issuer_signature_hash = decode_hash32(&private.issuer_signature_hash_b64)?;

    proving::generate_employment_proof(
        current_timestamp,
        &company_hash,
        &employment_type_hash,
        &employee_id_hash,
        private.start_date,
        private.end_date,
        private.salary,
        &position_hash,
        &credential_hash,
        &issuer_signature_hash,
        proving_key_bytes,
    )
}

fn decode_hash32(b64: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let bytes = base64::decode(b64)?;
    if bytes.len() != 32 {
        return Err(format!("expected 32 bytes, got {}", bytes.len()).into());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

fn success_response(result: ProofResult) -> *mut c_char {
    let resp = ProverResponse {
        ok: true,
        proof_base64: Some(base64::encode(&result.proof_bytes)),
        public_inputs: Some(
            result.public_inputs
                .into_iter()
                .map(|bytes| base64::encode(&bytes))
                .collect()
        ),
        error: None,
    };
    
    CString::new(serde_json::to_string(&resp).unwrap())
        .unwrap()
        .into_raw()
}

fn error_response(msg: &str) -> *mut c_char {
    let resp = ProverResponse {
        ok: false,
        proof_base64: None,
        public_inputs: None,
        error: Some(msg.to_string()),
    };
    
    CString::new(serde_json::to_string(&resp).unwrap())
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn free_rust_cstring(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

// JNI wrapper for Android
#[cfg(feature = "android")]
#[allow(non_snake_case)]
pub mod jni {
    use super::*;
    use jni::JNIEnv;
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;

    #[no_mangle]
    pub extern "system" fn Java_com_mobile_zk_ProverNative_generate_1proof_1json_1native(
        env: JNIEnv,
        _class: JClass,
        input: JString,
    ) -> jstring {
        let input_str: String = match env.get_string(input) {
            Ok(s) => s.into(),
            Err(_) => {
                let error = r#"{"ok":false,"error":"failed to get input string"}"#;
                return env.new_string(error).unwrap().into_inner();
            }
        };

        let c_input = match CString::new(input_str) {
            Ok(s) => s,
            Err(_) => {
                let error = r#"{"ok":false,"error":"failed to create C string"}"#;
                return env.new_string(error).unwrap().into_inner();
            }
        };

        let result_ptr = generate_proof_json(c_input.as_ptr());
        
        let result_str = unsafe {
            CStr::from_ptr(result_ptr).to_string_lossy().into_owned()
        };

        free_rust_cstring(result_ptr);

        env.new_string(result_str).unwrap().into_inner()
    }
}