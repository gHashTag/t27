# NOW -- a dispatch that can only fail, on a pipeline that is not broken (2026-09-06)

## a dispatch that can only fail, on a pipeline that is not broken (Refs #3316)

- release.yml was on the never-green list and should not have been. Run 33180327861 SUCCEEDED on 2026-08-28 publishing t27c 0.2.0. Of the recent failures, the 2026-08-29 one is a release whose tag names no product, which is the PRODUCT GATE doing its job.
- The one false red is a bare workflow_dispatch. On a dispatch github.event.release.tag_name is empty, preflight takes its catch-all branch and exits 1, and every publishing job is skipped. It cannot publish and cannot measure -- it can only fail.
- Removed rather than made to work. Giving preflight a tag input would make preflight SUCCEED on a dispatch, and preflight refusing is the structural reason a dispatch cannot reach a registry. That trades a structural guard for five if: conditions in front of live cargo publish and npm publish, on a pipeline that has already burned two permanent version numbers.
