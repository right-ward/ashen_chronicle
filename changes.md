# Ashen Chronicle legal fixes (copy-paste edition)

Each file below is under its repo path. Copy the contents into a file with that name. `LICENSE` and `LICENSES/Apache-2.0.txt` are the unmodified Apache-2.0 text from https://www.apache.org/licenses/LICENSE-2.0.txt, and `LICENSES/LicenseRef-Ashen-Chronicle-Content-1.1.txt` is a copy of `LICENSE-CONTENT`. `ci.patch` applies with `git apply` or `patch -p1`.

---

# Ashen Chronicle legal fixes: apply notes

Drafts only. Have an IP lawyer review before you monetize or submit to Play.

## What's here

```
files/                          mirrors the repo root
  LICENSE                       pristine Apache-2.0 (no Scope Notice, Appendix restored)
  LICENSE-CONTENT               Game Content License v1.1 (rewritten)
  NOTICE                        copyright + attribution + licensing map pointers
  THIRD-PARTY-NOTICES           replacement; now covers software, not just content
  REUSE.toml                    path -> license map that LICENSE-CONTENT s1.3 relies on
  LICENSES/                     Apache-2.0.txt + LicenseRef-...-1.1.txt (REUSE layout)
  legal/ANDROID-DEPENDENCIES.txt
  about.toml, about.hbs         cargo-about allow-list + template
  .github/actions/third-party-licenses/action.yml
ci.patch                        release workflows + .gitignore (dry-run tested)
```

## Apply

```sh
git apply ../ashen_chronicle-legal-fixes/ci.patch   # or: patch -p1 < ../.../ci.patch
grep -rn 'LEGAL NAME\|JURISDICTION' .               # fill every hit
cmp LICENSE-CONTENT LICENSES/LicenseRef-Ashen-Chronicle-Content-1.1.txt
```

The two copies of the content license must stay identical.

## Decisions I made (change any you disagree with)

1. **Subscriptions.** Your v1.0 listed subscriptions as allowed monetization
   but also banned gating Game Content behind payment. I resolved it toward
   the ban: subscriptions are fine only if they don't gate Game Content
   (s2.3, s4.1b).
2. **Paid mods.** A paid mod made only of your own material that doesn't copy
   or adapt Game Content is allowed (s7.2). Delete that sentence if you want
   paid mods barred.
3. **Gameplay footage** (streams, screenshots, reviews) needs no attribution (s3.4).
4. **App-store terms** don't breach the "no extra restrictions" rule if the
   license still travels with the content (s6.1c).
5. **Cure period:** 30 days, automatic reinstatement (s13).
6. **Versions:** v1.1 lets existing recipients keep v1.0 rights or elect v1.1 (s16).
7. **Developer docs** (README, docs/, ROADMAP) are Apache-2.0. Everything under
   `data/` is Game Content, including the mods README.
8. **Contributors** are named as co-copyright holders in REUSE.toml, matching
   CONTRIBUTING.md ("contributors retain copyright").

## Not verified (I had no network or Rust toolchain here)

- `reuse lint`: I only checked the globs with a script (all 92 files covered, no overlaps).
- `cargo about generate`: flags (`--locked`, `-o`) and template variables follow
  the cargo-about docs as I know them; run `cargo about generate --help` first.
  The first run may fail on a license missing from `about.toml` `accepted`: that
  is the gate working. Open the output and confirm it contains per-crate
  "Copyright" lines; MIT/BSD need those.
- **Bevy `default_font`** embeds a font. Historically Fira Mono under OFL-1.1.
  Check: `find ~/.cargo/registry/src -path '*bevy_text-0.19*' \( -name '*.ttf' -o -name 'LICENSE*' \)`.
  If it is OFL, add its copyright line and the OFL text to THIRD-PARTY-NOTICES.
- Cargo.toml has no `[patch]` section, so the "used unmodified" statement in
  THIRD-PARTY-NOTICES holds today. Keep it true or edit the text.

## Still open (not in this package)

- **CONTRIBUTING.md.** Highest remaining risk. It still has the dead "future
  versions" clause, a patent grant that differs from Apache s3, no explicit
  right for you to sell or relicense contributions, and weak assent. Needs your
  decision on relicensing and commercial rights first; then DCO or a CLA bot.
