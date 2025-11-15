package com.mobile.zk

import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors

object ProverNative {
    init {
        System.loadLibrary("prover") // libprover.so
    }

    // Extern JNI that returns a Java String (simpler). Alternatively return pointer and free via another JNI call.
    private external fun generate_proof_json_native(inputJson: String): String

    private val executor = Executors.newSingleThreadExecutor()

    fun generateProofAsync(inputJson: String): CompletableFuture<String> {
        return CompletableFuture.supplyAsync({
            // Heavy native call - runs on background thread
            generate_proof_json_native(inputJson)
        }, executor)
    }
}