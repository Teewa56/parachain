import type { Credential, ProofType, ZkProof } from '../../types/substrate';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import { blake2AsU8a } from '@polkadot/util-crypto';

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
    /**
     * Generate a zero-knowledge proof for selective disclosure
     */
    async generateProof(params: ProofGenerationParams): Promise<GeneratedProof> {
        const { credential, fieldsToReveal, proofType } = params;

        try {
            // Generate nonce for replay attack prevention
            const nonce = this.generateNonce();

            // Construct public inputs from revealed fields
            const publicInputs = this.constructPublicInputs(
                credential,
                fieldsToReveal,
                proofType
            );

            // Generate mock proof data (in production, this would call actual ZK circuit)
            const proofData = await this.generateProofData(
                credential,
                fieldsToReveal,
                publicInputs,
                nonce
            );

            // Create the ZK proof structure
            const proof: ZkProof = {
                proofType,
                proofData,
                publicInputs,
                credentialHash: credential.dataHash,
                createdAt: Math.floor(Date.now() / 1000),
                nonce,
            };

            // Create revealed fields map for display
            const revealedFields = this.extractRevealedFields(
                credential,
                fieldsToReveal
            );

            console.log('ZK proof generated successfully');
            console.log('Proof type:', proofType);
            console.log('Fields revealed:', fieldsToReveal.length);

            return {
                proof,
                revealedFields,
            };
        } catch (error) {
            console.error('ZK proof generation failed:', error);
            throw error;
        }
    }

    /**
     * Generate a unique nonce for replay attack prevention
     */
    private generateNonce(): string {
        const timestamp = Date.now();
        const random = Math.random();
        const data = new Uint8Array([
            ...new Uint8Array(new Float64Array([timestamp]).buffer),
            ...new Uint8Array(new Float64Array([random]).buffer),
        ]);
        
        const hash = blake2AsU8a(data, 256);
        return u8aToHex(hash);
    }

    /**
     * Construct public inputs for the ZK proof
     */
    private constructPublicInputs(
        credential: Credential,
        fieldsToReveal: number[],
        proofType: ProofType
    ): Uint8Array[] {
        const inputs: Uint8Array[] = [];

        // Add credential hash
        inputs.push(hexToU8a(credential.dataHash));

        // Add fields bitmap (which fields are revealed)
        const bitmap = this.createFieldsBitmap(fieldsToReveal);
        inputs.push(bitmap);

        // Add issuer DID
        inputs.push(hexToU8a(credential.issuer));

        // Add proof type hash
        const typeHash = blake2AsU8a(proofType, 256);
        inputs.push(typeHash);

        // Add timestamp
        const timestamp = Math.floor(Date.now() / 1000);
        const timestampBytes = new Uint8Array(32);
        new DataView(timestampBytes.buffer).setBigUint64(24, BigInt(timestamp), false);
        inputs.push(timestampBytes);

        return inputs;
    }

    /**
     * Create a bitmap representing which fields are revealed
     */
    private createFieldsBitmap(fieldsToReveal: number[]): Uint8Array {
        let bitmap = 0n;

        for (const fieldIndex of fieldsToReveal) {
            if (fieldIndex >= 64) {
                throw new Error('Field index too large (max 63)');
            }
            bitmap |= 1n << BigInt(fieldIndex);
        }

        const bytes = new Uint8Array(8);
        new DataView(bytes.buffer).setBigUint64(0, bitmap, true);
        return bytes;
    }

    /**
     * Generate proof data (mock implementation)
     * In production, this would call the actual Groth16 proving algorithm
     */
    private async generateProofData(
        credential: Credential,
        fieldsToReveal: number[],
        publicInputs: Uint8Array[],
        nonce: string
    ): Promise<Uint8Array> {
        // Combine all data for proof generation
        const dataToHash = new Uint8Array([
            ...hexToU8a(credential.dataHash),
            ...hexToU8a(credential.issuer),
            ...hexToU8a(credential.subject),
            ...new Uint8Array(fieldsToReveal),
            ...publicInputs.flat(),
            ...hexToU8a(nonce),
        ]);

        // Generate deterministic proof data
        // NOTE: In production, this would be replaced with actual Groth16 proof generation
        const proofHash = blake2AsU8a(dataToHash, 256);
        
        // Extend to 256 bytes (typical Groth16 proof size)
        const proofData = new Uint8Array(256);
        for (let i = 0; i < 256; i++) {
            proofData[i] = proofHash[i % 32] ^ (i % 256);
        }

        return proofData;
    }

    /**
     * Extract revealed field values for display
     */
    private extractRevealedFields(
        credential: Credential,
        fieldsToReveal: number[]
    ): Map<number, any> {
        const revealedFields = new Map<number, any>();

        // Mock field data extraction
        // In production, this would decrypt and parse the actual credential data
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

    /**
     * Get field names for a credential type
     */
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

    /**
     * Validate proof parameters before generation
     */
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

        const fieldNames = this.getFieldNamesForType(credential.credentialType);
        const invalidFields = fieldsToReveal.filter(
            (index) => index < 0 || index >= fieldNames.length
        );

        if (invalidFields.length > 0) {
            return {
                valid: false,
                error: `Invalid field indices: ${invalidFields.join(', ')}`,
            };
        }

        if (credential.status !== 'Active') {
            return {
                valid: false,
                error: `Credential is ${credential.status}, cannot generate proof`,
            };
        }

        // Check expiration
        if (credential.expiresAt > 0) {
            const now = Math.floor(Date.now() / 1000);
            if (now > credential.expiresAt) {
                return { valid: false, error: 'Credential has expired' };
            }
        }

        return { valid: true };
    }

    /**
     * Get available fields for a credential
     */
    getAvailableFields(credential: Credential): Array<{
        index: number;
        name: string;
        description: string;
    }> {
        const fieldNames = this.getFieldNamesForType(credential.credentialType);

        return fieldNames.map((name, index) => ({
            index,
            name,
            description: this.getFieldDescription(credential.credentialType, name),
        }));
    }

    /**
     * Get field description for display
     */
    private getFieldDescription(credentialType: string, fieldName: string): string {
        const descriptions: Record<string, Record<string, string>> = {
            Education: {
                institution: 'Name of educational institution',
                studentId: 'Student identification number',
                status: 'Current enrollment status',
                gpa: 'Grade point average',
                enrollmentDate: 'Date of enrollment',
                graduationDate: 'Expected or actual graduation date',
            },
            Health: {
                patientId: 'Patient identification number',
                vaccinationType: 'Type of vaccination received',
                vaccinationDate: 'Date of vaccination',
                expiryDate: 'Vaccination expiry date',
                provider: 'Healthcare provider name',
                batchNumber: 'Vaccine batch number',
            },
            Employment: {
                employeeId: 'Employee identification number',
                employer: 'Employer organization name',
                position: 'Job title or position',
                startDate: 'Employment start date',
                endDate: 'Employment end date (if applicable)',
                salary: 'Salary information',
            },
            Age: {
                birthYear: 'Year of birth',
                birthMonth: 'Month of birth',
                birthDay: 'Day of birth',
                ageThreshold: 'Age threshold for verification',
            },
            Address: {
                street: 'Street address',
                city: 'City',
                state: 'State or province',
                zipCode: 'Postal code',
                country: 'Country',
            },
        };

        const typeDescriptions = descriptions[credentialType];
        return typeDescriptions?.[fieldName] || `${fieldName} field`;
    }

    /**
     * Calculate estimated proof generation time
     */
    estimateProofGenerationTime(fieldsCount: number): number {
        // Base time + time per field (in milliseconds)
        const baseTime = 500;
        const timePerField = 100;
        return baseTime + fieldsCount * timePerField;
    }
}

export const zkProofService = new ZkProofService();
