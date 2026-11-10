# Sum

`sum_to` adds the numbers from 1 to `n`:

```mbt check
///|
test "sum_to in the docs" {
  inspect(@example.sum_to(4), content="10")
}
```
