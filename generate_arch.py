lines = []
lines.append('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 700">')
lines.append('  <style>')
lines.append('    @import url("https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&amp;display=swap");')
lines.append('    text { font-family: "Inter", sans-serif; }')
lines.append('    .node { fill: #1c1c1e; stroke: #3d3a39; stroke-width: 2; rx: 8; }')
lines.append('    .node-text { fill: #e5e5ea; font-size: 14px; font-weight: 500; }')
lines.append('    .node-highlight { fill: #0a0a0c; stroke: #00d992; stroke-width: 2; rx: 8; }')
lines.append('    .layer-box { fill: none; stroke: #2c2c2e; stroke-width: 2; stroke-dasharray: 6 4; rx: 12; }')
lines.append('    .layer-text { fill: #8e8e93; font-size: 16px; font-weight: 600; }')
lines.append('    .arrow { stroke: #00d992; stroke-width: 2; fill: none; marker-end: url(#arrowhead); }')
lines.append('    .arrow-dashed { stroke: #a1a1aa; stroke-width: 2; fill: none; stroke-dasharray: 6 4; marker-end: url(#arrowhead-gray); }')
lines.append('    .label-bg { fill: #000000; opacity: 0.8; rx: 4; }')
lines.append('    .label-text { fill: #a1a1aa; font-size: 12px; }')
lines.append('  </style>')
lines.append('  <defs>')
lines.append('    <marker id="arrowhead" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto">')
lines.append('      <path d="M 0 0 L 10 5 L 0 10 z" fill="#00d992" />')
lines.append('    </marker>')
lines.append('    <marker id="arrowhead-gray" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto">')
lines.append('      <path d="M 0 0 L 10 5 L 0 10 z" fill="#a1a1aa" />')
lines.append('    </marker>')
lines.append('  </defs>')
lines.append('  <rect width="100%" height="100%" fill="#050507"/>')

# Title
lines.append('  <text x="480" y="40" fill="#e5e5ea" font-size="20" font-weight="600" text-anchor="middle">Widget-RS Architecture</text>')

# Layer 1: App Layer
lines.append('  <rect class="layer-box" x="80" y="80" width="800" height="120" />')
lines.append('  <text class="layer-text" x="100" y="110">App Layer</text>')

lines.append('  <rect class="node-highlight" x="120" y="130" width="160" height="40" />')
lines.append('  <text class="node-text" x="200" y="155" text-anchor="middle">Main (main.rs)</text>')

lines.append('  <rect class="node" x="340" y="130" width="140" height="40" />')
lines.append('  <text class="node-text" x="410" y="155" text-anchor="middle">TrayIcon</text>')

lines.append('  <rect class="node" x="520" y="130" width="140" height="40" />')
lines.append('  <text class="node-text" x="590" y="155" text-anchor="middle">WindowManager</text>')

lines.append('  <rect class="node" x="700" y="130" width="140" height="40" />')
lines.append('  <text class="node-text" x="770" y="155" text-anchor="middle">PluginManager</text>')

# Layer 2: Plugin System
lines.append('  <rect class="layer-box" x="80" y="240" width="800" height="120" />')
lines.append('  <text class="layer-text" x="100" y="270">Plugin System</text>')

lines.append('  <rect class="node" x="700" y="290" width="140" height="40" rx="20" />')
lines.append('  <text class="node-text" x="770" y="315" text-anchor="middle">Plugin Trait</text>')

lines.append('  <rect class="node" x="180" y="290" width="140" height="40" />')
lines.append('  <text class="node-text" x="250" y="315" text-anchor="middle">StickyPlugin</text>')

lines.append('  <rect class="node" x="380" y="290" width="140" height="40" />')
lines.append('  <text class="node-text" x="450" y="315" text-anchor="middle">TodoPlugin</text>')

# Layer 3: Core & UI
lines.append('  <rect class="layer-box" x="80" y="400" width="800" height="120" />')
lines.append('  <text class="layer-text" x="100" y="430">Core &amp; UI</text>')

lines.append('  <rect class="node" x="240" y="450" width="180" height="40" />')
lines.append('  <text class="node-text" x="330" y="475" text-anchor="middle">widget-ui + GPUI</text>')

lines.append('  <rect class="node" x="540" y="450" width="180" height="40" />')
lines.append('  <text class="node-text" x="630" y="475" text-anchor="middle">widget-core</text>')

# Layer 4: System Native
lines.append('  <rect class="layer-box" x="80" y="560" width="800" height="100" />')
lines.append('  <text class="layer-text" x="100" y="590">System Native</text>')

lines.append('  <rect class="node-highlight" x="520" y="600" width="180" height="40" rx="20"/>')
lines.append('  <text class="node-text" x="610" y="625" text-anchor="middle" fill="#00d992">Edge Snapping (Win32)</text>')

# Arrows
# main -> tray
lines.append('  <path class="arrow-dashed" d="M 280,150 L 332,150" />')
# main -> wm
lines.append('  <path class="arrow-dashed" d="M 200,170 L 200,200 L 590,200 L 590,178" />')
# main -> pm
lines.append('  <path class="arrow-dashed" d="M 200,170 L 200,220 L 770,220 L 770,178" />')

# pm -> trait
lines.append('  <path class="arrow" d="M 770,170 L 770,282" />')
lines.append('  <rect class="label-bg" x="780" y="215" width="60" height="20" />')
lines.append('  <text class="label-text" x="810" y="229" text-anchor="middle">manages</text>')

# trait -> plugins
lines.append('  <path class="arrow-dashed" d="M 700,310 L 328,310" />')
lines.append('  <path class="arrow-dashed" d="M 700,310 L 528,310" />')

# plugins -> ui
lines.append('  <path class="arrow" d="M 250,330 L 250,442" />')
lines.append('  <path class="arrow" d="M 450,330 L 450,400 L 330,400 L 330,442" />')
lines.append('  <rect class="label-bg" x="190" y="375" width="50" height="20" />')
lines.append('  <text class="label-text" x="215" y="389" text-anchor="middle">renders</text>')

# core -> pm
lines.append('  <path class="arrow" d="M 630,450 L 630,360 L 820,360 L 820,170 L 810,170" />')
lines.append('  <rect class="label-bg" x="825" y="255" width="80" height="20" />')
lines.append('  <text class="label-text" x="865" y="269" text-anchor="middle">config state</text>')

# core -> ui
lines.append('  <path class="arrow-dashed" d="M 540,470 L 428,470" />')

# wm -> edge
lines.append('  <path class="arrow" d="M 590,170 L 590,592" />')
lines.append('  <rect class="label-bg" x="595" y="525" width="160" height="20" />')
lines.append('  <text class="label-text" x="675" y="539" text-anchor="middle">WM_WINDOWPOSCHANGING</text>')

lines.append('</svg>')

with open('assets/architecture.svg', 'w') as f:
    f.write('\n'.join(lines))
