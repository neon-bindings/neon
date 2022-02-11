import { expect } from "chai";
import BuildSettings from "../../../../cli/lib/build-settings";

describe("build settings", () => {
  const env = {
    npm_config_target: null,
    npm_config_arch: null,
    npm_config_target_arch: null,
    npm_config_disturl: null,
    npm_config_node_engine: null,
    npm_config_node_gyp: null,
    npm_config_nodedir: null,
    npm_config_platform: null,
    npm_config_runtime: null,
    npm_config_build_from_source: null,
    npm_config_devdir: null,
  };

  describe("match", () => {
    it("should not match build settings if nodeVersion is different", () => {
      const buildSettings = new BuildSettings("rustc-version", "v11.8.0", env);
      const otherSettings = new BuildSettings("rustc-version", "v10.8.0", env);
      expect(buildSettings.match(otherSettings)).equal(false);
    });

    it("should match build settings if nodeVersion is same", () => {
      const buildSettings = new BuildSettings("rustc-version", "v11.8.0", env);
      const otherSettings = new BuildSettings("rustc-version", "v11.8.0", env);
      expect(buildSettings.match(otherSettings)).equal(true);
    });

    it("should not match against current build settings when nodeVersion is null", () => {
      const buildSettings = BuildSettings.current();
      const otherSettings = new BuildSettings("rustc-version", null, env);
      expect(buildSettings.match(otherSettings)).equal(false);
    });
  });

  describe("serialize", () => {
    it("should serialize when nodeVersion is not set", () => {
      const settings = new BuildSettings("rustc-version", null, env);
      expect(settings.toJSON()).to.have.property("nodeVersion", null);
    });

    it("should serialize when nodeVersion is set", () => {
      const settings = new BuildSettings("rustc-version", "12.0.0", env);
      expect(settings.toJSON())
        .to.have.property("nodeVersion")
        .to.be.a("string");
    });
  });

  describe("current build settings", () => {
    it("should return values of expected types", () => {
      // @ts-ignore
      const { nodeVersion, env, rustc } = BuildSettings.current();
      expect(nodeVersion).to.be.a("string");
      expect(rustc).to.be.a("string");
      expect(env).to.deep.equal({
        npm_config_target: process.env.npm_config_target || null,
        npm_config_arch: process.env.npm_config_arch || null,
        npm_config_target_arch: process.env.npm_config_target_arch || null,
        npm_config_disturl: process.env.npm_config_disturl || null,
        npm_config_node_engine: process.env.npm_config_node_engine || null,
        npm_config_node_gyp: process.env.npm_config_node_gyp || null,
        npm_config_nodedir: process.env.npm_config_nodedir || null,
        npm_config_platform: process.env.npm_config_platform || null,
        npm_config_runtime: process.env.npm_config_runtime || null,
        npm_config_build_from_source:
          process.env.npm_config_build_from_source || null,
        npm_config_devdir: process.env.npm_config_devdir || null,
      });
    });
  });

  describe("get node version", () => {
    it("should return a string", () => {
      expect(BuildSettings.getNodeVersion()).to.be.a("string");
    });
  });

  describe("serialize and deserialize", () => {
    it("should be equal when nodeVersion is null", () => {
      expect(
        JSON.stringify(
          BuildSettings.fromJSON({
            rustc: "rustc-version",
            env,
          })
        )
      ).equal(
        JSON.stringify(new BuildSettings("rustc-version", null, env).toJSON())
      );
    });
  });
});
