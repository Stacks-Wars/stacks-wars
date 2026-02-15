/**
 * A secure external link component that opens URLs in a new tab.
 *
 * This component automatically applies security best practices by setting:
 * - `target="_blank"` to open links in a new tab
 * - `rel="noopener"` to prevent the new page from accessing the `window.opener` property,
 *   protecting against reverse tabnapping attacks
 * - `rel="noreferrer"` to prevent the browser from sending the referrer header,
 *   protecting user privacy and preventing referrer leakage
 *
 * @param {string} href - The URL to navigate to when the link is clicked
 * @param {React.ReactNode} children - The content to display inside the link (text, icons, etc.)
 *
 * @example
 * ```tsx
 * <ExternalLink href="https://example.com">
 *   Visit Example
 * </ExternalLink>
 * ```
 *
 * @security
 * The `noopener` attribute is critical for preventing malicious websites from
 * gaining partial access to the originating page via `window.opener`.
 * The `noreferrer` attribute prevents sensitive information in the URL from
 * being leaked to the external site.
 */
export function ExternalLink({
	href,
	children,
	className,
}: {
	href: string;
	children: React.ReactNode;
	className?: string;
}) {
	return (
		<a
			href={href}
			target="_blank"
			rel="noopener noreferrer"
			className={className}
		>
			{children}
		</a>
	);
}
