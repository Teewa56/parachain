import { generateProof as nativeGenerateProof } from '../../zk/prover';
import { provingKeyManager } from './provingKeys';
import type { Credential, ProofType, ZkProof } from '../../types/substrate';
import { blake2AsU8a, blake2AsHex } from '@polkadot/util-crypto';
import { u8aToHex, hexToU8a } from '@polkadot/util';

interface ProofGenerationParams {
    credential: Credential;
    fieldsToReveal: number[];
    proofType: ProofType;
}

interface GeneratedProof {
    proof: ZkProof;
    revealedFields: Map<number, any>;
}

class ZkProofService {
    async generateProof(params: ProofGenerationParams): Promise<GeneratedProof> {
        const { credential, fieldsToReveal, proofType } = params;

        try {
            // Get circuit ID
            const circuitId = this.mapProofTypeToCircuit(proofType);
            
            // Load proving key
            console.log(`Loading proving key for ${circuitId}...`);
            const provingKeyB64 = await provingKeyManager.getProvingKey(circuitId);
            
            // Prepare inputs based on proof type
            const { publicInputs, privateInputs } = await this.prepareInputs(
                proofType,
                credential,
                fieldsToReveal
            );
            
            // Generate nonce
            const nonce = this.generateNonce();
            
            console.log(`Generating ${circuitId} proof...`);
            
            // Call native prover
            const request = {
                circuit_id: circuitId,
                public_inputs: publicInputs,
                private_inputs: privateInputs,
                proving_key_b64: provingKeyB64,
            };
            
            const response = await nativeGenerateProof(request);
            
            if (!response.ok) {
                throw new Error(response.error || 'Proof generation failed');
            }
            
            console.log('✅ Proof generated successfully');
            
            // Parse response
            const proofData = Uint8Array.from(
                atob(response.proof_base64!),
                c => c.charCodeAt(0)
            );
            
            const publicInputsBytes = response.public_inputs!.map(b64 =>
                Uint8Array.from(atob(b64), c => c.charCodeAt(0))
            );
            
            const proof: ZkProof = {
                proofType,
                proofData,
                publicInputs: publicInputsBytes,
                credentialHash: credential.dataHash,
                createdAt: Math.floor(Date.now() / 1000),
                nonce,
            };
            
            const revealedFields = this.extractRevealedFields(credential, fieldsToReveal);
            
            return { proof, revealedFields };
        } catch (error) {
            console.error('Proof generation error:', error);
            throw error;
        }
    }
    
    private mapProofTypeToCircuit(proofType: ProofType): string {
        const mapping: Record<ProofType, string> = {
            AgeAbove: 'age_verification',
            StudentStatus: 'student_status',
            VaccinationStatus: 'vaccination_status',
            EmploymentStatus: 'employment_status',
            Custom: 'custom',
        };
        return mapping[proofType];
    }
    
    private async prepareInputs(
        proofType: ProofType,
        credential: Credential,
        fieldsToReveal: number[]
    ): Promise<{ publicInputs: any; privateInputs: string }> {
        switch (proofType) {
            case 'AgeAbove':
                return this.prepareAgeInputs(credential, fieldsToReveal);
            case 'StudentStatus':
                return this.prepareStudentInputs(credential, fieldsToReveal);
            case 'VaccinationStatus':
                return this.prepareVaccinationInputs(credential, fieldsToReveal);
            case 'EmploymentStatus':
                return this.prepareEmploymentInputs(credential, fieldsToReveal);
            default:
                throw new Error(`Unsupported proof type: ${proofType}`);
        }
    }
    
    private async prepareAgeInputs(
        credential: Credential,
        fieldsToReveal: number[]
    ): Promise<{ publicInputs: any; privateInputs: string }> {
        const currentTimestamp = Math.floor(Date.now() / 1000);
        const ageThreshold = 18; // 18 years
        
        // Hash credential type
        const credentialTypeHash = blake2AsU8a(credential.credentialType, 256);
        const credentialTypeHashB64 = btoa(String.fromCharCode(...credentialTypeHash));
        
        const publicInputs = {
            current_timestamp: currentTimestamp,
            age_threshold_years: ageThreshold,
            credential_type_hash_b64: credentialTypeHashB64,
        };
        
        // Parse private data from credential
        // In production, this would decrypt credential.dataHash
        const birthTimestamp = currentTimestamp - (25 * 365 * 24 * 3600); // Mock: 25 years old
        const credentialHash = hexToU8a(credential.dataHash);
        const issuerSignatureHash = hexToU8a(credential.signature);
        
        const privateInputs = JSON.stringify({
            birth_timestamp: birthTimestamp,
            credential_hash_b64: btoa(String.fromCharCode(...credentialHash)),
            issuer_signature_hash_b64: btoa(String.fromCharCode(...issuerSignatureHash)),
        });
        
        return { publicInputs, privateInputs };
    }
    
    private async prepareStudentInputs(
        credential: Credential,
        fieldsToReveal: number[]
    ): Promise<{ publicInputs: any; privateInputs: string }> {
        const currentTimestamp = Math.floor(Date.now() / 1000);
        
        // Mock institution hash (in production, parse from credential)
        const institutionName = "MIT";
        const institutionHash = blake2AsU8a(institutionName, 256);
        const institutionHashB64 = btoa(String.fromCharCode(...institutionHash));
        
        const statusActive = credential.status === 'Active';
        
        const publicInputs = {
            current_timestamp: currentTimestamp,
            institution_hash_b64: institutionHashB64,
            status_active: statusActive,
        };
        
        // Mock private inputs (parse from credential in production)
        const studentIdHash = blake2AsU8a("STUDENT123", 256);
        const enrollmentDate = currentTimestamp - (2 * 365 * 24 * 3600); // 2 years ago
        const expiryDate = currentTimestamp + (2 * 365 * 24 * 3600); // 2 years future
        const gpa = 350; // 3.5 GPA (stored as 350)
        const credentialHash = hexToU8a(credential.dataHash);
        const issuerSignatureHash = hexToU8a(credential.signature);
        
        const privateInputs = JSON.stringify({
            student_id_hash_b64: btoa(String.fromCharCode(...studentIdHash)),
            enrollment_date: enrollmentDate,
            expiry_date: expiryDate,
            gpa: gpa,
            credential_hash_b64: btoa(String.fromCharCode(...credentialHash)),
            issuer_signature_hash_b64: btoa(String.fromCharCode(...issuerSignatureHash)),
        });
        
        return { publicInputs, privateInputs };
    }
    
    private async prepareVaccinationInputs(
        credential: Credential,
        fieldsToReveal: number[]
    ): Promise<{ publicInputs: any; privateInputs: string }> {
        const currentTimestamp = Math.floor(Date.now() / 1000);
        
        // Mock vaccination type hash
        const vaccinationType = "COVID-19";
        const vaccinationTypeHash = blake2AsU8a(vaccinationType, 256);
        const vaccinationTypeHashB64 = btoa(String.fromCharCode(...vaccinationTypeHash));
        
        const minDosesRequired = 2;
        
        const publicInputs = {
            current_timestamp: currentTimestamp,
            vaccination_type_hash_b64: vaccinationTypeHashB64,
            min_doses_required: minDosesRequired,
        };
        
        // Mock private inputs
        const patientIdHash = blake2AsU8a("PATIENT456", 256);
        const vaccinationDate = currentTimestamp - (180 * 24 * 3600); // 6 months ago
        const expiryDate = credential.expiresAt;
        const dosesReceived = 2;
        const batchNumberHash = blake2AsU8a("BATCH789", 256);
        const credentialHash = hexToU8a(credential.dataHash);
        const issuerSignatureHash = hexToU8a(credential.signature);
        
        const privateInputs = JSON.stringify({
            patient_id_hash_b64: btoa(String.fromCharCode(...patientIdHash)),
            vaccination_date: vaccinationDate,
            expiry_date: expiryDate,
            doses_received: dosesReceived,
            batch_number_hash_b64: btoa(String.fromCharCode(...batchNumberHash)),
            credential_hash_b64: btoa(String.fromCharCode(...credentialHash)),
            issuer_signature_hash_b64: btoa(String.fromCharCode(...issuerSignatureHash)),
        });
        
        return { publicInputs, privateInputs };
    }
    
    private async prepareEmploymentInputs(
        credential: Credential,
        fieldsToReveal: number[]
    ): Promise<{ publicInputs: any; privateInputs: string }> {
        const currentTimestamp = Math.floor(Date.now() / 1000);
        
        // Mock company hash
        const companyName = "Tech Corp";
        const companyHash = blake2AsU8a(companyName, 256);
        const companyHashB64 = btoa(String.fromCharCode(...companyHash));
        
        const employmentType = "Full-time";
        const employmentTypeHash = blake2AsU8a(employmentType, 256);
        const employmentTypeHashB64 = btoa(String.fromCharCode(...employmentTypeHash));
        
        const publicInputs = {
            current_timestamp: currentTimestamp,
            company_hash_b64: companyHashB64,
            employment_type_hash_b64: employmentTypeHashB64,
        };
        
        // Mock private inputs
        const employeeIdHash = blake2AsU8a("EMP001", 256);
        const startDate = currentTimestamp - (3 * 365 * 24 * 3600); // 3 years ago
        const endDate = 0; // Still employed
        const salary = 7500000; // $75,000 in cents
        const positionHash = blake2AsU8a("Software Engineer", 256);
        const credentialHash = hexToU8a(credential.dataHash);
        const issuerSignatureHash = hexToU8a(credential.signature);
        
        const privateInputs = JSON.stringify({
            employee_id_hash_b64: btoa(String.fromCharCode(...employeeIdHash)),
            start_date: startDate,
            end_date: endDate,
            salary: salary,
            position_hash_b64: btoa(String.fromCharCode(...positionHash)),
            credential_hash_b64: btoa(String.fromCharCode(...credentialHash)),
            issuer_signature_hash_b64: btoa(String.fromCharCode(...issuerSignatureHash)),
        });
        
        return { publicInputs, privateInputs };
    }
    
    private generateNonce(): string {
        const timestamp = Date.now();
        const random = Math.random();
        const data = new Uint8Array([
            ...new Uint8Array(new Float64Array([timestamp]).buffer),
            ...new Uint8Array(new Float64Array([random]).buffer),
        ]);
        return u8aToHex(blake2AsU8a(data, 256));
    }
    
    private extractRevealedFields(
        credential: Credential,
        fieldsToReveal: number[]
    ): Map<number, any> {
        const revealedFields = new Map<number, any>();
        const fieldNames = this.getFieldNamesForType(credential.credentialType);

        fieldsToReveal.forEach((index) => {
            if (index < fieldNames.length) {
                revealedFields.set(index, {
                    name: fieldNames[index],
                    value: `[Field ${index} Value]`,
                });
            }
        });

        return revealedFields;
    }
    
    private getFieldNamesForType(credentialType: string): string[] {
        const fieldMappings: Record<string, string[]> = {
            Education: [
                'institution',
                'studentId',
                'status',
                'gpa',
                'enrollmentDate',
                'graduationDate',
            ],
            Health: [
                'patientId',
                'vaccinationType',
                'vaccinationDate',
                'expiryDate',
                'provider',
                'batchNumber',
            ],
            Employment: [
                'employeeId',
                'employer',
                'position',
                'startDate',
                'endDate',
                'salary',
            ],
            Age: [
                'birthYear',
                'birthMonth',
                'birthDay',
                'ageThreshold',
            ],
            Address: [
                'street',
                'city',
                'state',
                'zipCode',
                'country',
            ],
            Custom: [
                'field0',
                'field1',
                'field2',
                'field3',
                'field4',
            ],
        };

        return fieldMappings[credentialType] || fieldMappings.Custom;
    }
    
    validateProofParams(params: ProofGenerationParams): {
        valid: boolean;
        error?: string;
    } {
        const { credential, fieldsToReveal, proofType } = params;

        if (!credential) {
            return { valid: false, error: 'Credential is required' };
        }

        if (!fieldsToReveal || fieldsToReveal.length === 0) {
            return { valid: false, error: 'At least one field must be revealed' };
        }

        if (fieldsToReveal.length > 50) {
            return { valid: false, error: 'Too many fields selected (max 50)' };
        }

        if (credential.status !== 'Active') {
            return {
                valid: false,
                error: `Credential is ${credential.status}, cannot generate proof`,
            };
        }

        if (credential.expiresAt > 0) {
            const now = Math.floor(Date.now() / 1000);
            if (now > credential.expiresAt) {
                return { valid: false, error: 'Credential has expired' };
            }
        }

        return { valid: true };
    }
    
    estimateProofGenerationTime(fieldsCount: number): number {
        // Real ZK proof generation is more complex
        // Groth16 typically takes 2-5 seconds on mobile
        return 3000; // 3 seconds baseline
    }
}

export const zkProofService = new ZkProofService();