# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state
### v0.50.3: Responsive Bevy presentation for Android landscape
- Made the shared Bevy screen root use viewport-relative spacing so the same layout scales across phone, tablet, and resizable desktop landscape resolutions.
- Made shared panels flexible so dashboard, navigation, lifecycle, record, interaction, and result content can shrink into the available viewport while retaining internal scrolling for longer content.
- Made shared labels and button text wrap within their available width instead of allowing long content to force the layout wider than the screen.
- Converted shared title/body typography and the progression feedback overlay to viewport-relative font and spacing units while preserving the existing dark, text-first visual identity.
- Kept interactive controls at a 48px logical minimum height to provide a stable touch target while responsive spacing and typography adapt around them.
- Reused the same shared presentation implementation for desktop and Android; Android remains landscape-only through the existing activity configuration.

## Next

Continue the v0.50.x Android presentation/input work with #220, #221, and #222, followed by #223, #224, and #225.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.