#!/usr/bin/env python3
# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Create identity.onnx (one input, same shape output) for godot-onnx sample."""
import sys
from onnx import helper, TensorProto
from onnx.checker import check_model

def main():
    in_x = helper.make_tensor_value_info("x", TensorProto.FLOAT, [3])
    out_y = helper.make_tensor_value_info("y", TensorProto.FLOAT, [3])
    node = helper.make_node("Identity", inputs=["x"], outputs=["y"])
    graph = helper.make_graph([node], "identity", [in_x], [out_y])
    # IR 11 and opset 11 for ort compatibility (max supported IR version: 11)
    model = helper.make_model(
        graph,
        ir_version=11,
        opset_imports=[helper.make_opsetid("", 11)],
    )
    check_model(model)
    out_path = "identity.onnx"
    with open(out_path, "wb") as f:
        f.write(model.SerializeToString())
    print(f"Wrote {out_path}")

if __name__ == "__main__":
    main()
