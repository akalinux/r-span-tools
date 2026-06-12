#!/bin/bash
RUSTFLAGS="-C instrument-coverage"  cargo test --tests
llvm-profdata merge -sparse default_*.profraw -o json5format.profdata

llvm-cov report \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-C instrument-coverage" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --instr-profile=json5format.profdata --summary-only
