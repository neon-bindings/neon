---
title: Error handling
description: How Rust Result values map to JavaScript exceptions in Neon.
status: todo
---

This page explains the model Neon uses for errors: how a Rust `Result::Err` becomes a thrown JavaScript exception, how thrown JS exceptions surface back into Rust, and what role types like `extract::Error` play in keeping that bridge ergonomic.
