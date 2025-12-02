import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import ThemeProvider from "@/components/theme/theme-provider";
import { Toaster } from "@/components/ui/sonner";
import { SpeedInsights } from "@vercel/speed-insights/next";

const geistSans = Geist({
	variable: "--font-geist-sans",
	subsets: ["latin"],
});

const geistMono = Geist_Mono({
	variable: "--font-geist-mono",
	subsets: ["latin"],
});

export const metadata: Metadata = {
	metadataBase: new URL("https://stackswars.com"),
	title: {
		default: "Stacks Wars - Web3 Gaming Platform on Stacks Blockchain",
		template: "%s | Stacks Wars",
	},
	description:
		"The ultimate Web3 gaming platform on Stacks blockchain. Compete in skill-based games like Lexi Wars, join tournaments, and earn real STX rewards. Fair play, transparent gaming, endless competition.",
	keywords: [
		"stacks blockchain",
		"web3 gaming",
		"blockchain games",
		"crypto gaming",
		"STX rewards",
		"play to earn",
		"skill-based gaming",
		"multiplayer games",
		"blockchain tournaments",
		"gaming platform",
		"stacks ecosystem",
		"word games",
		"strategy games",
		"competitive gaming",
		"lexi wars",
		"casino games",
		"GameFi",
		"GameFi on Stacks",
		"decentralized gaming",
		"Bitcoin L2 gaming",
		"smart contract games",
		"competitive esports",
	],
	authors: [{ name: "SWE Hashiras", url: "" }],
	creator: "SWE Hashiras",
	publisher: "Stacks Wars",
	formatDetection: {
		email: false,
		address: false,
		telephone: false,
	},
	robots: {
		index: true,
		follow: true,
		googleBot: {
			index: true,
			follow: true,
			"max-video-preview": -1,
			"max-image-preview": "large",
			"max-snippet": -1,
		},
	},
	verification: {
		// Add these when you have them
		// google: 'your-google-site-verification',
		// other: {
		//   'msvalidate.01': 'your-bing-verification',
		// },
	},
	openGraph: {
		type: "website",
		locale: "en_US",
		url: "https://stackswars.com",
		siteName: "Stacks Wars",
		title: "Stacks Wars - Web3 Gaming Platform on Stacks Blockchain",
		description:
			"Compete in skill-based Web3 games on Stacks blockchain. Join tournaments, play Lexi Wars, and earn real STX rewards. Fair, transparent, and competitive gaming for everyone.",
		images: [
			{
				//url: "https://stackswars.com/og-image.png",
				url: "/logo.webp",
				width: 1200,
				height: 630,
				alt: "Stacks Wars - Web3 Gaming Platform",
				type: "image/webp",
			},
			{
				//url: "https://stackswars.com/og-image-square.png",
				url: "/logo.webp",
				width: 600,
				height: 600,
				alt: "Stacks Wars Logo",
				type: "image/webp",
			},
		],
	},
	twitter: {
		card: "summary_large_image",
		site: "@stackswars",
		creator: "@stackswars",
		title: "Stacks Wars - Web3 Gaming Platform",
		description:
			"🎮 Skill-based Web3 gaming on Stacks blockchain\n🏆 Compete in tournaments & earn STX rewards\n🎯 Fair play, transparent gaming\n🔥 Join the competition now!",
		images: {
			//url: "https://stackswars.com/twitter-image.png",
			url: "/logo.webp",
			alt: "Stacks Wars - Web3 Gaming Platform",
		},
	},
	icons: {
		icon: [
			{ url: "/favicon.ico", sizes: "any" },
			{ url: "/favicon-16x16.png", sizes: "16x16", type: "image/png" },
			{ url: "/favicon-32x32.png", sizes: "32x32", type: "image/png" },
		],
		apple: [
			{
				url: "/apple-touch-icon.png",
				sizes: "180x180",
				type: "image/png",
			},
		],
		other: [
			{
				rel: "android-chrome-192x192",
				url: "/android-chrome-192x192.png",
			},
			{
				rel: "android-chrome-512x512",
				url: "/android-chrome-512x512.png",
			},
		],
	},
	manifest: "/manifest.json",
	applicationName: "Stacks Wars",
	category: "Gaming",
	classification: "Gaming Platform",
	referrer: "origin-when-cross-origin",
	alternates: {
		canonical: "https://stackswars.com",
		languages: {
			"en-US": "https://stackswars.com",
			// Add other languages when available
			// 'es-ES': 'https://stackswars.com/es',
		},
	},
	other: {
		"apple-mobile-web-app-capable": "yes",
		"apple-mobile-web-app-status-bar-style": "default",
		"apple-mobile-web-app-title": "Stacks Wars",
		"mobile-web-app-capable": "yes",
		"msapplication-TileColor": "#000000",
		//"msapplication-config": "/browserconfig.xml",
	},
};

export default function RootLayout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<html lang="en" suppressHydrationWarning>
			<body
				className={`${geistSans.variable} ${geistMono.variable} antialiased`}
			>
				<ThemeProvider
					attribute="class"
					defaultTheme="system"
					enableSystem
					disableTransitionOnChange={false}
				>
					<div className="flex flex-col min-h-screen">{children}</div>
					<Toaster richColors position="top-right" />
				</ThemeProvider>
				<SpeedInsights />
			</body>
		</html>
	);
}
