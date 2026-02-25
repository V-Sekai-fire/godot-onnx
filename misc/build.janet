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
(var build-opts @{})
(var build-plat nil)
(var build-ext nil)
(var build-lib-so nil)
(var build-target-dir nil)
(var build-src-path nil)
(var build-src-alt nil)

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
    (def cwd (or (os/cwd) "."))
    (def src-abs (string cwd "/" src))
    (def dest-abs (string cwd "/" dest))
    # On Windows use copy; on Linux/macOS/BSD/posix use cp (os/which can be :posix on some runners)
    (def mode (if (= (os/which) :windows)
      (string "copy /Y \"" (string/replace-all "/" "\\" src-abs) "\" \"" (string/replace-all "/" "\\" dest-abs) "\"")
      (string "cp \"" src-abs "\" \"" dest-abs "\"")))
    (when (not (string? mode))
      (eprintf "copy-file: mode is not a string (os/which=%q)\n" (os/which))
      (os/exit 1))
    (def ret (os/shell mode))
    (when (not= ret 0)
      (eprintf "copy failed: %q -> %q (exit %q)\n" src dest ret)
      (os/exit ret))
    (printf "Copied to %s\n" dest)
    true))

(defn main []
  (do
    (def raw-args (dyn :args))
    (set build-opts (parse-args (if raw-args (tuple/slice raw-args 1) (tuple))))
    (set build-plat (os/which))
    (set build-ext (or (case build-plat :windows "dll" :macos "dylib" :linux "so") "so"))
    (set build-lib-so (string "lib" lib-name "." build-ext))
    (set build-target-dir (string "target/release"))
    (set build-src-path (string build-target-dir "/" build-lib-so))
    (set build-src-alt (string build-target-dir "/" lib-name "." build-ext))

    (when (not (get build-opts :skip-test))
      (printf "Running tests...\n")
      (def st (run-cargo "test"))
      (when (not= st 0) (os/exit st)))

    (when (not (get build-opts :skip-build))
      (printf "Building Godot ONNX GDExtension (float)...\n")
      (def st (run-cargo "build" "--release"))
      (when (not= st 0) (os/exit st)))

    (os/mkdir dest-dir)
    (def copied
      (or
        (copy-file build-src-path (string dest-dir "/" build-lib-so))
        (copy-file build-src-alt (string dest-dir "/" build-lib-so))))
    (when (not copied)
      (eprintf "Built library not found (tried %s or %s)\n" build-src-path build-src-alt)
      (os/exit 1))

    (when (and (= build-plat :macos) (os/stat (string dest-dir "/" build-lib-so)))
      (printf "Code signing GDExtension library for macOS...\n")
      (run "codesign" "--force" "--sign" "-" (string dest-dir "/" build-lib-so)))

    (when (and (not (get build-opts :skip-doubles)) (os/getenv "GODOT4_BIN"))
      (printf "Building Godot ONNX GDExtension (doubles)...\n")
      (def st (run-cargo "build" "--release" "--no-default-features" "--features" "double-precision"))
      (when (= st 0)
        (def dest-doubles (string dest-dir "/lib" lib-name "_doubles." build-ext))
        (def got (or
          (copy-file build-src-path dest-doubles)
          (copy-file build-src-alt dest-doubles)))
        (when (and got (= build-plat :macos))
          (run "codesign" "--force" "--sign" "-" dest-doubles)))
      (when (not= st 0)
        (printf "Doubles build failed. Float build is ready.\n")))
    (when (and (not (get build-opts :skip-doubles)) (not (os/getenv "GODOT4_BIN")))
      (printf "Doubles build skipped (set GODOT4_BIN to a double-precision Godot binary to build).\n"))

    (os/exit 0)))

(main)
