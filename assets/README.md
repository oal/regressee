# Regressee branding

- `logo.png`: user-supplied original for light backgrounds.
- `logo-dark.png`: user-supplied original for dark backgrounds.
- `brand/logo-light.png` and `brand/logo-dark.png`: padding-cropped, downsampled
  copies embedded into the native application. No recoloring or artwork changes.
- `brand/provenance.json`: origins, source/output hashes and exact transforms.

The dark mark sits beside **Regressee** in the navy navigation rail. The light
mark accompanies **About Regressee** on white. Neither is a replacement for the
readable app name. The original files remain untouched.

To regenerate the UI copies after replacing an original:

```sh
python3 scripts/prepare-brand-assets.py
```

This developer-only preparation tool requires Pillow; shipping the app does not.
Logos are embedded at compile time, so no asset directory or network access is
needed at runtime.
