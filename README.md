# Stateless Blockchain Experiment

This repository implements a UTXO-based stateless blockchain on Substrate using an RSA accumulator. In this scheme,
validators only need to track a single accumulator value and users only need to store their own UTXOs and membership
proofs. Unless a data service provider is used, users must constantly watch for updates to the accumulator in order to
update their proofs. This particular implementation includes batching and aggregation techniques from this paper:
https://eprint.iacr.org/2018/1188.pdf.

This repository is experimental and is not meant to be used in production. The design choices made in this runtime
are impractical from both a security and usability standpoint. Additionally, the following code has not been checked for
correctness nor has been optimized for efficiency.
