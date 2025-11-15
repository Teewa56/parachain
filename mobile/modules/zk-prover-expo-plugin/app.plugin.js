const { withDangerousMod, withPlugins } = require('@expo/config-plugins');

function withZKProverAndroid(config, options = {}) {
  return withDangerousMod(config, [
    'android',
    async (config) => {
      // copy .so files and update settings.gradle / build.gradle if needed
      // This plugin expects you run build scripts before eas build
      return config;
    },
  ]);
}

function withZKProverIOS(config, options = {}) {
  return withDangerousMod(config, [
    'ios',
    async (config) => {
      // Add rust-prover.xcframework to Xcode project via plugin patches or instruct manual step
      return config;
    }
  ]);
}

module.exports = function withZKProver(config, options) {
  config = withPlugins(config, [
    [withZKProverAndroid, options],
    [withZKProverIOS, options]
  ]);
  return config;
};
