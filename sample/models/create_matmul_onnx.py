#!/usr/bin/env python3
# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Create matmul.onnx (two inputs A, B; one output Y) for godot-onnx sample."""
import sys
from onnx import helper, TensorProto
from onnx.checker import check_model

def main():
    a = helper.make_tensor_value_info("A", TensorProto.FLOAT, [2, 3])
    b = helper.make_tensor_value_info("B", TensorProto.FLOAT, [3, 2])
    out = helper.make_tensor_value_info("Y", TensorProto.FLOAT, [2, 2])
    node = helper.make_node("MatMul", inputs=["A", "B"], outputs=["Y"])
    graph = helper.make_graph([node], "matmul", [a, b], [out])
    model = helper.make_model(graph)
    check_model(model)
    out_path = "matmul.onnx"
    with open(out_path, "wb") as f:
        f.write(model.SerializeToString())
    print(f"Wrote {out_path}")

if __name__ == "__main__":
    main()
