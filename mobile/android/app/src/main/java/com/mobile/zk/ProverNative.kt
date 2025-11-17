package com.mobile.zk

import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors

object ProverNative {
    init {
        System.loadLibrary("prover")
    }

    private external fun generate_proof_json_native(inputJson: String): String
    private external fun free_rust_cstring(ptr: Long)

    private val executor = Executors.newSingleThreadExecutor()

    fun generateProofAsync(inputJson: String): CompletableFuture<String> {
        return CompletableFuture.supplyAsync({
            generate_proof_json_native(inputJson)
        }, executor)
    }
}