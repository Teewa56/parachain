import Foundation

@objc(ZKProverModule)
class ZKProverModule: NSObject {
    
    @objc
    func generateProof(
        _ inputJson: String,
        resolver resolve: @escaping RCTPromiseResolveBlock,
        rejecter reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async {
            guard let cString = inputJson.cString(using: .utf8) else {
                reject("INVALID_INPUT", "Failed to convert input to C string", nil)
                return
            }
            
            let resultPtr = generate_proof_json(cString)
            
            guard let resultCStr = resultPtr else {
                reject("PROVER_ERROR", "Null pointer returned from Rust", nil)
                return
            }
            
            let resultString = String(cString: resultCStr)
            free_rust_cstring(resultPtr)
            
            resolve(resultString)
        }
    }
    
    @objc
    static func requiresMainQueueSetup() -> Bool {
        return false
    }
}

// C function declarations from Rust
@_cdecl("generate_proof_json")
func generate_proof_json(_ input: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_cdecl("free_rust_cstring")
func free_rust_cstring(_ ptr: UnsafeMutablePointer<CChar>?)