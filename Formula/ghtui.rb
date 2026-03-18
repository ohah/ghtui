class Ghtui < Formula
  desc "A comprehensive GitHub TUI built with Rust and ratatui"
  homepage "https://github.com/ohah/ghtui"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/ohah/ghtui/releases/download/v#{version}/ghtui-aarch64-apple-darwin.tar.gz"
      # sha256 "PLACEHOLDER"
    else
      url "https://github.com/ohah/ghtui/releases/download/v#{version}/ghtui-x86_64-apple-darwin.tar.gz"
      # sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/ohah/ghtui/releases/download/v#{version}/ghtui-aarch64-unknown-linux-gnu.tar.gz"
      # sha256 "PLACEHOLDER"
    else
      url "https://github.com/ohah/ghtui/releases/download/v#{version}/ghtui-x86_64-unknown-linux-gnu.tar.gz"
      # sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "ghtui"
  end

  test do
    assert_match "ghtui", shell_output("#{bin}/ghtui --version")
  end
end
