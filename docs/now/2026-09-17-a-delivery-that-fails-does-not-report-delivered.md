# NOW -- A delivery that fails does not report delivered (2026-09-17)

## What was read

- render-job-run: sendCallback caught the axios error and returned. The callback step COMPLETED on a dead URL, the try/catch in render.ts never fired, and the run reported a delivered render no client heard about.
- morph-images-generate: video_url is a path on the worker. It went to sendVideo and sendDocument as a bare string, the call failed, and the fallback sent the user that path as a download link while the step returned delivered.
- morph-images-generate SIDE_EFFECTS said charges-balance; the file imports getUserBalance only and deducts nothing.

## What the specs say now

- render-job-run NOTE: sendCallback rethrows; after RETRIES the run fails and onFailure tells the admin. The mkdir -p item of the plan is recorded as not needed: Windows host, ssh.exec resolves on any exit code.
- morph-images-generate SIDE_EFFECTS drops charges-balance; NOTE records the { source } delivery, the rethrow, and that the paid Kling calls stay unmetered.

## Not verified

- No live render or morph run was made; the deployed build was not exercised.

## Where the code goes

- 999-multibots-telegraf, branch `delivery-fixes`: renderSteps.ts, render.ts, morphImages.ts, functions.manifest.json, test render/deliveryIsNotSwallowed.test.ts citing these specs.
