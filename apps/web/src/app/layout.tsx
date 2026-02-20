import type { Metadata } from "next";
import "./globals.css";
import localFont from "next/font/local";
import { Provider } from "@/app/provider";
import { Toaster } from "@/components/ui/sonner";

const neueMontreal = localFont({
	src: [
		{
			path: "./fonts/NeueMontreal-Regular.woff2",
			weight: "400",
			style: "normal",
		},
		{
			path: "./fonts/NeueMontreal-Medium.woff2",
			weight: "500",
			style: "normal",
		},
		{
			path: "./fonts/NeueMontreal-Bold.woff2",
			weight: "700",
			style: "normal",
		},
	],
	variable: "--font-neue-montreal",
});
export const metadata: Metadata = {
	metadataBase: new URL("https://stackswars.com"),
	title: {
		default: "Stacks Wars - Real-Time Competitive Gaming Platform",
		template: "%s | Stacks Wars",
	},
	description:
		"Stacks Wars is a real-time multiplayer gaming platform where players compete in skill-based games while securely pooling and distributing rewards on-chain using Stacks.",
	keywords: [
		"Stacks Wars",
		"blockchain gaming",
		"web3 games",
		"real-time multiplayer",
		"competitive gaming",
		"skill-based games",
		"on-chain rewards",
		"Stacks blockchain",
		"play to compete",
		"crypto gaming",
	],
	authors: [{ name: "Stacks Wars" }],
	creator: "Stacks Wars",
	publisher: "Stacks Wars",
	formatDetection: {
		email: false,
		address: false,
		telephone: false,
	},
	openGraph: {
		type: "website",
		locale: "en_US",
		url: "https://stackswars.com",
		siteName: "Stacks Wars",
		title: "Stacks Wars - Real-Time Competitive Gaming",
		description:
			"Compete in multiplayer games with transparent, on-chain rewards. Stacks Wars combines real-time gameplay with trustless payouts on Stacks.",
		images: [
			{
				url: "/logo.webp",
				width: 1200,
				height: 630,
				alt: "Stacks Wars - Real-Time Competitive Gaming",
			},
		],
	},
	twitter: {
		card: "summary_large_image",
		title: "Stacks Wars - Real-Time Competitive Gaming",
		description:
			"Skill-based multiplayer games with transparent, on-chain rewards. Built on Stacks.",
		creator: "@stackswars",
		images: ["/logo.webp"],
	},
	icons: {
		icon: [
			{ url: "/favicon.ico" },
			{ url: "/favicon-16x16.png", sizes: "16x16", type: "image/png" },
			{ url: "/favicon-32x32.png", sizes: "32x32", type: "image/png" },
		],
		apple: [
			{ url: "/apple-icon.png" },
			{ url: "/apple-icon-72x72.png", sizes: "72x72", type: "image/png" },
			{
				url: "/apple-icon-114x114.png",
				sizes: "114x114",
				type: "image/png",
			},
		],
	},
	manifest: "/site.webmanifest",
	applicationName: "Stacks Wars",
	category: "Gaming",
};

export default function RootLayout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<html lang="en">
			<body className={`${neueMontreal.variable} font-neue antialiased`}>
				<Provider>
					<Toaster position="top-center" />
					{children}
				</Provider>
			</body>
		</html>
	);
}
