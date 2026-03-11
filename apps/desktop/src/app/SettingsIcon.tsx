import type { SVGProps } from 'react'

export function SettingsIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      <path d="M3 17h6v2H3v-2zm0-6h10v2H3v-2zm0-6h14v2H3V5zm16 12v2h2v-2h-2zm-2-2h6v6h-6v-6zm2-8h2V5h-2v2zm-2-2h6v6h-6V5z" />
    </svg>
  )
}
