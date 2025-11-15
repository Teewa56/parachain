use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};
use crate::proving;
use zeroize::Zeroize;

#[derive(Deserialize)]
struct ProverRequest {
    // Define the inputs the JS/Native side will send
    // Keep payload compact (field elements, base64 private blobs, circuit id, etc.)
    circuit_id: String,
    public_inputs: Vec<String>,
    private_inputs_b64: String, // base64 encoded binary secret for Rust to decode
    options: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ProverResponse {
    ok: bool,
    proof_base64: Option<String>,
    public_inputs: Option<Vec<String>>,
    error: Option<String>,
}

/// SAFETY: returns a heap allocated C string which the caller MUST free by calling `free_rust_cstring(ptr)`
#[no_mangle]
pub extern "C" fn generate_proof_json(request_ptr: *const c_char) -> *mut c_char {
    // Null pointer check
    if request_ptr.is_null() {
        let err = ProverResponse { ok:false, proof_base64: None, public_inputs: None, error: Some("null request".to_string()) };
        return CString::new(serde_json::to_string(&err).unwrap()).unwrap().into_raw();
    }

    // Convert C string to Rust &str
    let c_str = unsafe { CStr::from_ptr(request_ptr) };
    let req_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            let err = ProverResponse { ok:false, proof_base64: None, public_inputs: None, error: Some("invalid utf8".to_string()) };
            return CString::new(serde_json::to_string(&err).unwrap()).unwrap().into_raw();
        }
    };

    // Parse request JSON
    let req: ProverRequest = match serde_json::from_str(req_str) {
        Ok(r) => r,
        Err(e) => {
            let err = ProverResponse { ok:false, proof_base64: None, public_inputs: None, error: Some(format!("json parse error: {}", e)) };
            return CString::new(serde_json::to_string(&err).unwrap()).unwrap().into_raw();
        }
    };

    // IMPORTANT: decode private inputs securely into Vec<u8> and zeroize after use
    let mut private_bytes = match base64::decode(&req.private_inputs_b64) {
        Ok(b) => b,
        Err(e) => {
            let err = ProverResponse { ok:false, proof_base64: None, public_inputs: None, error: Some(format!("base64 decode error: {}", e)) };
            return CString::new(serde_json::to_string(&err).unwrap()).unwrap().into_raw();
        }
    };

    // Call the prover. This function should implement heavy work and return proof bytes + public inputs
    let result = proving::generate_proof(
        &req.circuit_id,
        &req.public_inputs,
        &mut private_bytes,
        req.options
    );

    // Zeroize private_bytes immediately
    private_bytes.zeroize();

    // Build response
    let resp_json = match result {
        Ok((proof_bytes, public_inputs)) => {
            let proof_b64 = base64::encode(&proof_bytes);
            let resp = ProverResponse {
                ok: true,
                proof_base64: Some(proof_b64),
                public_inputs: Some(public_inputs),
                error: None,
            };
            serde_json::to_string(&resp).unwrap()
        },
        Err(e) => {
            let resp = ProverResponse { ok:false, proof_base64: None, public_inputs: None, error: Some(format!("{}", e)) };
            serde_json::to_string(&resp).unwrap()
        }
    };

    CString::new(resp_json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn free_rust_cstring(ptr: *mut c_char) {
    if ptr.is_null() { return; }
    unsafe {
        CString::from_raw(ptr); // deallocates
    }
}