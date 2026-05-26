class Aget < Formula
  desc "Local-first auth-aware agent web context CLI"
  homepage "https://github.com/nicolaslara/aget"
  url "https://github.com/nicolaslara/aget/releases/download/v0.1.0/aget-v0.1.0-aarch64-apple-darwin.tar.gz"
  sha256 "b02a6f348a5adfbfd280b7b915cc485095dd4f67990c04d700871e8b37763182"
  license "MIT"
  version "0.1.0"

  depends_on :macos
  depends_on arch: :arm64

  def install
    bin.install "aget"
    pkgshare.install "README.md"
    pkgshare.install "LICENSE"
    pkgshare.install "skills"
    pkgshare.install "scripts"
  end

  def caveats
    <<~EOS
      Install the global Codex skill with:
        #{pkgshare}/scripts/install-codex-skill.sh --copy --force

      Restart Codex after installing or replacing the global skill.
    EOS
  end

  test do
    assert_match "aget 0.1.0", shell_output("#{bin}/aget --version")
    output = shell_output("AGET_HOME=#{testpath}/aget-home #{bin}/aget --envelope json doctor --quick")
    assert_match '"ok":true', output
  end
end
