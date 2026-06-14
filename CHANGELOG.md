# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-06-13

### Added 

 - This file CHANGELOG.md
 - OverlapIterWithOverlaps object with: constructors, unit tests, and examples, and docs.
 - OverlapIter::reset(&mut self), resets the iterator.
 - OverlapIter::next_overlaps(&mut self) which returns a tuple wich contians the next range and an instance of OverlapIterWithOverlaps.
 - OverlapIter::next_back_overlaps(&mut self) which returns a tuple which contains the next range and an instance of OverlapIterWithOverlaps.

## [1.0.0] - 2026-06-12

1.0.0 contains Cargo.toml cleanups along with documentation updates.

## [0.1.0] - 2026-06-12

0.1.0 was the inital release.

