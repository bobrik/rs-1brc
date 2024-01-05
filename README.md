# One billion row challenge in rust

See: https://github.com/gunnarmorling/1brc

This is my rust version of the solution.

## Running

### Input data

I don't have enough RAM to fit the whole dataset and enough CPUs to make it run
in a reasonable time, so I'm using a 100 million row version. The input was
generated from the `1brc` repo and you can get it here:

* https://r2.ivan.computer/1brc/100m.txt.zstd

Download and extract it first:

```
curl https://r2.ivan.computer/1brc/100m.txt.zstd | zstd -d > 100m.txt
```

There's also a 10 million row version if you are extra lazy:

* https://r2.ivan.computer/1brc/10m.txt.zstd

### Running the code

To run the actual code with the input:

```
cargo run --release -- 100m.txt
```

## Speed

At the time of writing, the fastest code in the upstream repository was this:

* https://github.com/gunnarmorling/1brc/blob/4af3253d53/src/main/java/dev/morling/onebrc/CalculateAverage_spullara.java

Running it in my VM gives me:

```
5.46user 0.08system 0:00.81elapsed 679%CPU (0avgtext+0avgdata 1406588maxresident)k
0inputs+64outputs (0major+32191minor)pagefaults 0swaps
```

The code in this repo is slightly faster:

```
5.76user 0.07system 0:00.79elapsed 737%CPU (0avgtext+0avgdata 1348992maxresident)k
448inputs+0outputs (2major+21477minor)pagefaults 0swaps
```

All tests are running in a Linux VM on my M1 Macbook Air.
