use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};

use crate::proving::{self, ProofResult};
use base64::{Engine as _, engine::general_purpose};

#[derive(Deserialize)]
struct ProverRequest {
    circuit_id: String,
    public_inputs_b64: String,
    private_inputs_b64: String,
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
    if request_ptr.is_null() {
        return error_response("null request pointer");
    }

    let c_str = unsafe { CStr::from_ptr(request_ptr) };
    let req_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return error_response("invalid utf8"),
    };

    let req: ProverRequest = match serde_json::from_str(req_str) {
        Ok(r) => r,
        Err(e) => return error_response(&format!("json parse error: {}", e)),
    };

    // Decode with Engine
    let public_input = match general_purpose::STANDARD.decode(&req.public_inputs_b64) {
        Ok(b) => b,
        Err(e) => return error_response(&format!("public input decode error: {}", e)),
    };

    let private_input = match general_purpose::STANDARD.decode(&req.private_inputs_b64) {
        Ok(b) => b,
        Err(e) => return error_response(&format!("private input decode error: {}", e)),
    };

    // Generate proof
    let result = proving::generate_sp1_proof(&req.circuit_id, public_input, private_input);

    match result {
        Ok(proof_result) => success_response(proof_result),
        Err(e) => error_response(&e.to_string()),
    }
}

fn success_response(result: ProofResult) -> *mut c_char {
    let resp = ProverResponse {
        ok: true,
        proof_base64: Some(general_purpose::STANDARD.encode(&result.proof_bytes)),
        public_inputs: Some(
            result.public_inputs
                .into_iter()
                .map(|bytes| general_purpose::STANDARD.encode(&bytes))
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

// JNI wrapper for Android (unchanged, works with new proving)
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