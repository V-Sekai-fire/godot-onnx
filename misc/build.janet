# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Single script for building godot-onnx: use from GitHub Actions and locally.
# Usage: janet misc/build.janet [--skip-test] [--skip-build] [--skip-doubles] [--ci]
#   --ci       : CI mode (copy into sample/addons/godot-onnx for artifact)
#   --skip-test: do not run cargo test
#   --skip-build: do not run cargo build --release (only copy existing artifact)
#   --skip-doubles: do not build double-precision variant even if GODOT4_BIN is set

(def lib-name "godot_onnx")
(def dest-dir "sample/addons/godot-onnx")

(defn usage []
  (print "Usage: janet misc/build.janet [--skip-test] [--skip-build] [--skip-doubles] [--ci]")
  (os/exit 1))

(defn parse-args [args]
  (var skip-test false)
  (var skip-build false)
  (var skip-doubles false)
  (var ci false)
  (each arg args
    (cond
      (= arg "--skip-test") (set skip-test true)
      (= arg "--skip-build") (set skip-build true)
      (= arg "--skip-doubles") (set skip-doubles true)
      (= arg "--ci") (set ci true)
      (= arg "--help") (usage)
      true (do (print (string "Unknown option: " arg)) (usage))))
  @{:skip-test skip-test :skip-build skip-build :skip-doubles skip-doubles :ci ci})

(defn run [& args]
  (def status (os/execute (tuple ;args) :p))
  status)

(defn run-cargo [& args]
  (apply run (array "cargo" ;args)))

(defn ensure-dir [path]
  (unless (os/stat path) (os/mkdir path)))

(defn copy-file [src dest]
  (when (os/stat src)
    (ensure-dir "sample")
    (ensure-dir "sample/addons")
    (ensure-dir dest-dir)
    (def mode (case (os/which)
      :windows (string "copy /Y \"" src "\" \"" dest "\"")
      true (string "cp " src " " dest)))
    (def ret (os/shell mode))
    (when (not= ret 0)
      (eprintf "copy failed: %q -> %q (exit %q)\n" src dest ret)
      (os/exit ret))
    (printf "Copied to %s\n" dest)
    true))

(defn main []
  (def opts (parse-args (rest (os/args))))
  (def plat (os/which))
  # RUNNER_OS in CI is Windows, Linux, macOS
  (def runner-os (os/getenv "RUNNER_OS"))
  (def os-key (if runner-os
    (string/lower runner-os)
    (case plat :windows "windows" :macos "macos" true "linux")))
  (def ext (case plat :windows "dll" :macos "dylib" true "so"))
  (def lib-so (string "lib" lib-name "." ext))
  (def target-dir (string "target/release"))
  (def src-path (string target-dir "/" lib-so))
  (def src-alt (string target-dir "/" lib-name "." ext))

  (when (not (get opts :skip-test))
    (printf "Running tests...\n")
    (def st (run-cargo "test"))
    (when (not= st 0) (os/exit st)))

  (when (not (get opts :skip-build))
    (printf "Building Godot ONNX GDExtension (float)...\n")
    (def st (run-cargo "build" "--release"))
    (when (not= st 0) (os/exit st)))

  (os/mkdir dest-dir)
  (def copied
    (or
      (copy-file src-path (string dest-dir "/" lib-so))
      (copy-file src-alt (string dest-dir "/" lib-so))))
  (when (not copied)
    (eprintf "Built library not found (tried %s or %s)\n" src-path src-alt)
    (os/exit 1))

  (when (and (= plat :macos) (os/stat (string dest-dir "/" lib-so)))
    (printf "Code signing GDExtension library for macOS...\n")
    (run "codesign" "--force" "--sign" "-" (string dest-dir "/" lib-so))))

  (when (and (not (get opts :skip-doubles)) (os/getenv "GODOT4_BIN"))
    (printf "Building Godot ONNX GDExtension (doubles)...\n")
    (def st (run-cargo "build" "--release" "--no-default-features" "--features" "double-precision"))
    (when (= st 0)
      (def dest-doubles (string dest-dir "/lib" lib-name "_doubles." ext))
      (def got (or
        (copy-file src-path dest-doubles)
        (copy-file src-alt dest-doubles)))
      (when (and got (= plat :macos))
        (run "codesign" "--force" "--sign" "-" dest-doubles)))
    (when (not= st 0)
      (printf "Doubles build failed. Float build is ready.\n")))
  (when (and (not (get opts :skip-doubles)) (not (os/getenv "GODOT4_BIN")))
    (printf "Doubles build skipped (set GODOT4_BIN to a double-precision Godot binary to build).\n"))

  (os/exit 0))

(main)
