# Mona

CU-optimised, autism-driven Solana Router. It doesn't get any better.

```
                              o8%8888,
                            o88%8888888.
                           8'-    -:8888b
                          8'         8888
                         d8.-=. ,==-.:888b
                         >8 `~` :`~' d8888
                         88         ,88888
                         88b. `-~  ':88888
                         888b ~==~ .:88888
                         88888o--:':::8888
                         `88888| :::' 8888b
                         8888^^'       8888b
                        d888           ,%888b.
                       d88%            %%%8--'-.
                      /88:.__ ,       _%-' ---  -
                          '''::===..-'   =  --.
```

---

The routing relies on a flat account layout, each adapter owns its full account slice.

Instruction data (after selector):

```
   flags=0x01 (chained)                       flags=0x02 (split)
   ─────────────────────────────────          ─────────────────────────────────
   ┌──────────────────────┐                   ┌──────────────────────┐
   │ flags           (1B) │                   │ flags           (1B) │
   │ amount_in       (8B) │                   │ num_steps       (1B) │
   │ amount_out_min  (8B) │                   ├──────────────────────┤
   │ num_steps       (1B) │                   │ step[0].dex     (1B) │
   ├──────────────────────┤                   │ step[0].a_to_b  (1B) │
   │ step[0].dex     (1B) │                   │ step[0].amt_in  (8B) │
   │ step[0].a_to_b  (1B) │                   │ step[0].out_min (8B) │
   │ ...                  │                   │ ...                  │
   └──────────────────────┘                   └──────────────────────┘
   header: 18B, step: 2B each                 header: 2B, step: 18B each

   accounts:
   ┌──────────────────────────────────────────────┐
   │ [0]  payer (signer, writable)                │
   │ [1..N₀]  hop 0 remaining accounts            │
   │ [N₀..N₁] hop 1 remaining accounts            │
   │ ...                                          │
   └──────────────────────────────────────────────┘
```
