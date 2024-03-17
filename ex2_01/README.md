# 演習問題 2-1

## 実行方法

パッケージのダウンロードが必要かも

```bash
$ julia --project=./ src/ex2_01.jl
```

## 実行結果

```bash
9×3 DataFrame
 Row │ name      method     price
     │ String15  String15   Int64
─────┼────────────────────────────
   1 │ もも肉    生肉           8
   2 │ もも肉    燻製/通常     14
   3 │ もも肉    燻製/超過     11
   4 │ バラ肉    生肉           4
   5 │ バラ肉    燻製/通常     12
   6 │ バラ肉    燻製/超過      7
   7 │ 肩肉      生肉           4
   8 │ 肩肉      燻製/通常     13
   9 │ 肩肉      燻製/超過      9
Press any key:
9×4 DataFrame
 Row │ name      method     price  quantity
     │ String15  String15   Int64  GenericV…
─────┼─────────────────────────────────────────
   1 │ もも肉    生肉           8  quantity[1]
   2 │ もも肉    燻製/通常     14  quantity[2]
   3 │ もも肉    燻製/超過     11  quantity[3]
   4 │ バラ肉    生肉           4  quantity[4]
   5 │ バラ肉    燻製/通常     12  quantity[5]
   6 │ バラ肉    燻製/超過      7  quantity[6]
   7 │ 肩肉      生肉           4  quantity[7]
   8 │ 肩肉      燻製/通常     13  quantity[8]
   9 │ 肩肉      燻製/超過      9  quantity[9]
Press any key:
Max 8 quantity[1] + 14 quantity[2] + 11 quantity[3] + 4 quantity[4] + 12 quantity[5] + 7 quantity[6] + 4 quantity[7] + 13 quantity[8] + 9 quantity[9]
Subject to

 quantity[1] + quantity[2] + quantity[3] ≤ 480
 quantity[4] + quantity[5] + quantity[6] ≤ 400
 quantity[7] + quantity[8] + quantity[9] ≤ 230
 quantity[2] + quantity[5] + quantity[8] ≤ 420
 quantity[3] + quantity[6] + quantity[9] ≤ 250
 quantity[1] ≥ 0
 quantity[2] ≥ 0
 quantity[3] ≥ 0
 quantity[4] ≥ 0
 quantity[5] ≥ 0
 quantity[6] ≥ 0
 quantity[7] ≥ 0
 quantity[8] ≥ 0
 quantity[9] ≥ 0

Press any key: Running HiGHS 1.7.0 (git hash: 50670fd4c): Copyright (c) 2024 HiGHS under MIT licence terms
Coefficient ranges:
  Matrix [1e+00, 1e+00]
  Cost   [4e+00, 1e+01]
  Bound  [0e+00, 0e+00]
  RHS    [2e+02, 5e+02]
Presolving model
5 rows, 9 cols, 15 nonzeros  0s
5 rows, 9 cols, 15 nonzeros  0s
Presolve : Reductions: rows 5(-0); columns 9(-0); elements 15(-0) - Not reduced
Problem not reduced by presolve: solving the LP
Using EKK dual simplex solver - serial
  Iteration        Objective     Infeasibilities num(sum)
          0    -8.1999930746e+01 Ph1: 5(15); Du: 9(81.9999) 0s
         10     1.0910000000e+04 Pr: 0(0) 0s
Model   status      : Optimal
Simplex   iterations: 10
Objective value     :  1.0910000000e+04
HiGHS run time      :          0.00
Press any key:
9×4 DataFrame
 Row │ name      method     price  quantity
     │ String15  String15   Int64  Float64
─────┼──────────────────────────────────────
   1 │ もも肉    生肉           8     440.0
   2 │ もも肉    燻製/通常     14       0.0
   3 │ もも肉    燻製/超過     11      40.0
   4 │ バラ肉    生肉           4       0.0
   5 │ バラ肉    燻製/通常     12     400.0
   6 │ バラ肉    燻製/超過      7       0.0
   7 │ 肩肉      生肉           4       0.0
   8 │ 肩肉      燻製/通常     13      20.0
   9 │ 肩肉      燻製/超過      9     210.0
```
