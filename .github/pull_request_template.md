See `CONTRIBUTING.md`, `docs/maintenance-policy.md`, and `docs/compatibility-update-policy.md` for contributor workflow, release expectations, compatibility-update cadence, MSRV policy, and breaking-change guidance.

## Summary
- What problem does this PR solve?
- What is the high-level approach?

## Related Issues
- Closes #<issue-number>
- Related to #<issue-number>

## Changes
- [ ] API behavior changes
- [ ] Type/model changes
- [ ] Docs/examples updates
- [ ] CI/release changes

## Validation
- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --test unit`
- [ ] `cargo test --test integration -- --nocapture` (if secrets are available)

## Breaking Changes
- [ ] No breaking changes
- [ ] Breaking changes (describe below)

## Notes for Reviewers
- Any caveats, assumptions, or follow-up work.
