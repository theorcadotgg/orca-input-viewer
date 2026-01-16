export const overlayTemplate = `
  <div class="controller-stage">
    <svg viewBox="-2 175 335.2 200" aria-label="Orca controller overlay" role="img">
      <rect x="2" y="186" width="325" height="182" rx="16" fill="rgba(90, 130, 170, 0.22)" />
      <image href="assets/ORCATOPBLANKTEMPLATE-Edge_Cuts.svg" x="0" y="0" width="603.4278" height="370.0780" />

      <g data-analog-index="0">
        <path class="oblong" d="M124.6254 228.8973 A12 12 0 1 0 101.1499 223.9075 L94.7046 254.23 A12 12 0 1 0 118.1802 259.2199 Z" />
        <text class="oblong-label" x="110" y="244" text-anchor="middle">--</text>
        <text class="oblong-value" x="110" y="255" text-anchor="middle">0.00</text>
      </g>
      <g data-analog-index="1">
        <path class="oblong" d="M98.7154 213.6873 A12 12 0 1 0 75.2399 208.6975 L68.7946 239.02 A12 12 0 1 0 92.2702 244.0099 Z" />
        <text class="oblong-label" x="84" y="228" text-anchor="middle">--</text>
        <text class="oblong-value" x="84" y="239" text-anchor="middle">0.00</text>
      </g>
      <g data-analog-index="2">
        <path class="oblong" d="M68.8654 217.0673 A12 12 0 1 0 45.3899 212.0775 L38.9446 242.4 A12 12 0 1 0 62.4202 247.3899 Z" />
        <text class="oblong-label" x="54" y="230" text-anchor="middle">--</text>
        <text class="oblong-value" x="54" y="241" text-anchor="middle">0.00</text>
      </g>
      <g data-analog-index="3">
        <path class="oblong" d="M141.611 311.0539 A12 12 0 0 0 121.7141 297.6333 L104.3791 323.3334 A12 12 0 0 0 124.276 336.7541 Z" />
        <text class="oblong-label" x="123" y="318" text-anchor="middle">--</text>
        <text class="oblong-value" x="123" y="329" text-anchor="middle">0.00</text>
      </g>
      <g data-analog-index="4">
        <path class="oblong" d="M309.868 229.2229 A12 12 0 1 0 286.2326 233.3905 L291.6156 263.9195 A12 12 0 1 0 315.251 259.752 Z" />
        <text class="oblong-label" x="300" y="248" text-anchor="middle">--</text>
        <text class="oblong-value" x="300" y="259" text-anchor="middle">0.00</text>
      </g>

      <g data-digital-index="0">
        <circle class="btn-circle" cx="219.6450" cy="229.0637" r="11.5" />
        <text class="btn-label" x="219.6450" y="232" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="1">
        <circle class="btn-circle" cx="224.9800" cy="294.6787" r="11.5" />
        <text class="btn-label" x="224.9800" y="298" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="2">
        <circle class="btn-circle" cx="21.7350" cy="256.7337" r="11.5" />
        <text class="btn-label" x="21.7350" y="260" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="3">
        <circle class="btn-circle" cx="178.5000" cy="251.4037" r="11.5" />
        <text class="btn-label" x="178.5000" y="254" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="4">
        <circle class="btn-circle" cx="265.1800" cy="214.2387" r="11.5" />
        <text class="btn-label" x="265.1800" y="218" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="5">
        <circle class="btn-circle" cx="199.0000" cy="339.7187" r="11.5" />
        <text class="btn-label" x="199.0000" y="343" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="6">
        <circle class="btn-circle" cx="185.9950" cy="317.1937" r="11.5" />
        <text class="btn-label" x="185.9950" y="320" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="7">
        <circle class="btn-circle" cx="198.9800" cy="294.6787" r="11.5" />
        <text class="btn-label" x="198.9800" y="298" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="8">
        <circle class="btn-circle" cx="212.0000" cy="317.1987" r="11.5" />
        <text class="btn-label" x="212.0000" y="320" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="9">
        <circle class="btn-circle" cx="244.5700" cy="237.1287" r="11.5" />
        <text class="btn-label" x="244.5700" y="240" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="10">
        <circle class="btn-circle" cx="270.6300" cy="239.8687" r="11.5" />
        <text class="btn-label" x="270.6300" y="243" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="11">
        <circle class="btn-circle" cx="224.9800" cy="254.6787" r="11.5" />
        <text class="btn-label" x="224.9800" y="258" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="12">
        <circle class="btn-circle" cx="165.6650" cy="229.0637" r="11.5" />
        <text class="btn-label" x="165.6650" y="232" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="13">
        <circle class="btn-circle" cx="239.1200" cy="211.4987" r="11.5" />
        <text class="btn-label" x="239.1200" y="214" text-anchor="middle">--</text>
      </g>
      <g data-digital-index="14">
        <circle class="btn-circle" cx="152.5000" cy="251.4787" r="11.5" />
        <text class="btn-label" x="152.5000" y="254" text-anchor="middle">--</text>
      </g>
    </svg>
  </div>
`;
