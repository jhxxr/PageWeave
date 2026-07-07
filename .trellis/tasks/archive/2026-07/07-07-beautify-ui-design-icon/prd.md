# Beautify UI and design icon

## Goal

Enhance the visual design of the PageWeave application to match a modern, premium aesthetic, and deploy the new custom application icon across all platforms.

## Requirements

1. **Custom Branding and Styling**:
   - Modern color palette (indigo/violet primary with teal accents) configured via global Ant Design tokens.
   - Elegant typography with Google Fonts ("Outfit" and "Inter").
   - Sleek styling utilities (glassmorphism, clean dark/light mode transitions) implemented in a new global `src/index.css`.
   
2. **Layout & Navigation**:
   - Floating sidebar with polished padding, custom logo header, and smooth menu selection animations.
   
3. **Refined Interfaces**:
   - **TranslatePage**: Redesigned drag-and-drop zone with animated glow borders, card-grid options layout, and a styled developer console for translations.
   - **ProviderPage & ParamsPage**: Clean card layouts, custom switches, improved tables, and forms.
   - **TasksPage & SettingsPage**: Refined lists, control fields, and action buttons.

4. **App Icon Assets**:
   - Save the custom generated icon to the public folder.
   - Create a Python script to resize the logo into all required Tauri format sizes inside `src-tauri/icons`.

## Acceptance Criteria

- [ ] `npm run build` compiles with no TypeScript or Vite build errors.
- [ ] Global styling integrates with both dark and light modes.
- [ ] Sidebar displays the new PageWeave icon logo and brand typography.
- [ ] Drag-and-drop area displays hover visual micro-animations.
- [ ] Monospace log viewer displays styled outputs with professional styling.
- [ ] Icon generation script successfully updates all Tauri icon files in `src-tauri/icons/`.
