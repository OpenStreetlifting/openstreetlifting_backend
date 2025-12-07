# Extractors

An Extractor, is a program that may contain a preprocessor, and a LLM call,
with goal to extract meet/competition data from a structured/unstructured file.
The idea is to pass raw data to a Large Language Model, for it to create a [canonical](./canonical_format.md) representation.
Then, the canonical file is verified, and added to the database of competitions.
