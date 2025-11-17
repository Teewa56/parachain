const { withDangerousMod, withPlugins } = require('@expo/config-plugins');

function withZKProverAndroid(config, options = {}) {
  return withDangerousMod(config, [
    'android',
    async (config) => {
      return config;
    },
  ]);
}

function withZKProverIOS(config, options = {}) {
  return withDangerousMod(config, [
    'ios',
    async (config) => {
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
