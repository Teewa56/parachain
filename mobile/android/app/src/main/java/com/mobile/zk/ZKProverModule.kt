package com.mobile.zk

import com.facebook.react.bridge.*
import com.facebook.react.module.annotations.ReactModule
import java.util.concurrent.CompletableFuture

@ReactModule(name = ZKProverModule.NAME)
class ZKProverModule(reactContext: ReactApplicationContext) : ReactContextBaseJavaModule(reactContext) {
    companion object {
        const val NAME = "ZKProverModule"
    }

    override fun getName(): String = NAME

    @ReactMethod
    fun generateProof(inputJson: String, promise: Promise) {
        // Call native async
        try {
            val fut: CompletableFuture<String> = ProverNative.generateProofAsync(inputJson)
            fut.whenComplete { res, err ->
                if (err != null) {
                    promise.reject("PROVER_ERROR", err.message)
                } else {
                    promise.resolve(res)
                }
            }
        } catch (e: Exception) {
            promise.reject("PROVER_ERROR", e.message)
        }
    }
}