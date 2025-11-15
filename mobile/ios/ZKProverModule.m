#import <React/RCTBridgeModule.h>

@interface RCT_EXTERN_MODULE(ZKProverModule, NSObject)
RCT_EXTERN_METHOD(generateProof:(NSString *)inputJson resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject)
@end
