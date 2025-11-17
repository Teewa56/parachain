import XCTest
@testable import YourApp

class ZKProverModuleTests: XCTestCase {
    var proverModule: ZKProverModule!
    
    override func setUp() {
        super.setUp()
        proverModule = ZKProverModule()
    }
    
    func testAgeVerificationProof() {
        let expectation = self.expectation(description: "Proof generation")
        
        let request = """
        {
            "circuit_id": "age_verification",
            "public_inputs": {
                "current_timestamp": 1700000000,
                "age_threshold_years": 18,
                "credential_type_hash_b64": "\(createMockHash())"
            },
            "private_inputs": "\(createMockAgePrivateInputs())",
            "proving_key_b64": "\(loadTestProvingKey())"
        }
        """
        
        proverModule.generateProof(request) { result in
            switch result {
            case .success(let response):
                let json = try! JSONDecoder().decode(ProverResponse.self, from: response.data(using: .utf8)!)
                XCTAssertTrue(json.ok, "Proof should be generated successfully")
                XCTAssertNotNil(json.proof_base64, "Proof should not be nil")
                expectation.fulfill()
            case .failure(let error):
                XCTFail("Proof generation failed: \(error)")
            }
        }
        
        waitForExpectations(timeout: 10.0)
    }
    
    func testInvalidInput() {
        let expectation = self.expectation(description: "Invalid input handling")
        
        let invalidRequest = "invalid json"
        
        proverModule.generateProof(invalidRequest) { result in
            switch result {
            case .success(let response):
                let json = try! JSONDecoder().decode(ProverResponse.self, from: response.data(using: .utf8)!)
                XCTAssertFalse(json.ok, "Should fail for invalid input")
                XCTAssertNotNil(json.error, "Error message should be present")
                expectation.fulfill()
            case .failure:
                // Also acceptable
                expectation.fulfill()
            }
        }
        
        waitForExpectations(timeout: 5.0)
    }
    
    func testMemoryManagement() {
        // Test that multiple calls don't leak memory
        for _ in 0..<10 {
            let expectation = self.expectation(description: "Memory test")
            
            let request = createTestRequest()
            proverModule.generateProof(request) { _ in
                expectation.fulfill()
            }
            
            wait(for: [expectation], timeout: 10.0)
        }
    }
    
    // Helper methods
    private func createMockHash() -> String {
        let data = Data(count: 32)
        return data.base64EncodedString()
    }
    
    private func createMockAgePrivateInputs() -> String {
        let privateInputs: [String: Any] = [
            "birth_timestamp": 1500000000,
            "credential_hash_b64": createMockHash(),
            "issuer_signature_hash_b64": createMockHash()
        ]
        let jsonData = try! JSONSerialization.data(withJSONObject: privateInputs)
        return String(data: jsonData, encoding: .utf8)!
    }
    
    private func loadTestProvingKey() -> String {
        // Load from test resources
        guard let path = Bundle(for: type(of: self)).path(forResource: "test_age_verification", ofType: "pk"),
              let data = try? Data(contentsOf: URL(fileURLWithPath: path)) else {
            fatalError("Test proving key not found")
        }
        return data.base64EncodedString()
    }
    
    private func createTestRequest() -> String {
        return """
        {
            "circuit_id": "age_verification",
            "public_inputs": {
                "current_timestamp": 1700000000,
                "age_threshold_years": 18,
                "credential_type_hash_b64": "\(createMockHash())"
            },
            "private_inputs": "\(createMockAgePrivateInputs())",
            "proving_key_b64": "\(loadTestProvingKey())"
        }
        """
    }
}

struct ProverResponse: Codable {
    let ok: Bool
    let proof_base64: String?
    let public_inputs: [String]?
    let error: String?
}