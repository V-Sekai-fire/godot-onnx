#!/usr/bin/env python3
# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Create benchmark.onnx: same I/O as matmul (A [2,3], B [3,2] -> Y [2,2]) but does 4 MatMuls in sequence.
Run this model in a loop and measure time; CPU is much slower than accelerated, so you can tell the difference."""
from onnx import helper, TensorProto
from onnx.checker import check_model

def main():
    a = helper.make_tensor_value_info("A", TensorProto.FLOAT, [2, 3])
    b = helper.make_tensor_value_info("B", TensorProto.FLOAT, [3, 2])
    out = helper.make_tensor_value_info("Y", TensorProto.FLOAT, [2, 2])
    # Constant 2x2 matrices (identity so output is still A@B)
    w_vals = [1.0, 0.0, 0.0, 1.0]
    W1 = helper.make_tensor("W1", TensorProto.FLOAT, [2, 2], w_vals)
    W2 = helper.make_tensor("W2", TensorProto.FLOAT, [2, 2], w_vals)
    W3 = helper.make_tensor("W3", TensorProto.FLOAT, [2, 2], w_vals)
    nodes = [
        helper.make_node("MatMul", inputs=["A", "B"], outputs=["T1"]),
        helper.make_node("MatMul", inputs=["T1", "W1"], outputs=["T2"]),
        helper.make_node("MatMul", inputs=["T2", "W2"], outputs=["T3"]),
        helper.make_node("MatMul", inputs=["T3", "W3"], outputs=["Y"]),
    ]
    graph = helper.make_graph(
        nodes, "benchmark", [a, b], [out], initializer=[W1, W2, W3]
    )
    model = helper.make_model(
        graph,
        ir_version=11,
        opset_imports=[helper.make_opsetid("", 11)],
    )
    check_model(model)
    out_path = "benchmark.onnx"
    with open(out_path, "wb") as f:
        f.write(model.SerializeToString())
    print(f"Wrote {out_path}")

if __name__ == "__main__":
    main()
