import Link from "next/link";
import {
	Swords,
	BookOpen,
	Code2,
	Gamepad2,
	Trophy,
	Wallet,
	Server,
	Globe,
} from "lucide-react";

const features = [
	{
		icon: Gamepad2,
		title: "Competitive Gaming",
		description:
			"Real-time multiplayer games with turn-based and elimination mechanics.",
	},
	{
		icon: Wallet,
		title: "On-Chain Wagering",
		description:
			"STX and fungible token vaults with smart contract escrow and prize distribution.",
	},
	{
		icon: Trophy,
		title: "Seasons & Leaderboards",
		description:
			"Climb the ranks with Wars Points, track stats, and compete in seasons.",
	},
	{
		icon: Server,
		title: "Plugin Architecture",
		description:
			"Extensible game engine system — add new games with a Rust engine and React plugin.",
	},
];

const quickLinks = [
	{
		href: "/docs",
		label: "Documentation",
		description: "Game rules, lobbies, smart contracts",
		icon: BookOpen,
	},
	{
		href: "/docs/dev/local-setup",
		label: "Dev Setup",
		description: "Run Stacks Wars locally",
		icon: Code2,
	},
	{
		href: "/docs/dev/building-a-game",
		label: "Build a Game",
		description: "Create your own game engine",
		icon: Gamepad2,
	},
	{
		href: "/docs/dev/api-reference",
		label: "API Reference",
		description: "HTTP & WebSocket endpoints",
		icon: Globe,
	},
];

export default function HomePage() {
	return (
		<div className="flex flex-1 flex-col">
			{/* Hero */}
			<section className="flex flex-col items-center justify-center gap-6 px-6 py-24 text-center md:py-32">
				<div className="bg-fd-muted inline-flex items-center gap-2 rounded-full px-4 py-1.5 text-sm font-medium">
					<Swords className="h-4 w-4" />
					Competitive On-Chain Gaming
				</div>
				<h1 className="max-w-3xl text-4xl font-bold tracking-tight md:text-6xl">
					Stacks Wars
				</h1>
				<p className="text-fd-muted-foreground max-w-2xl text-lg md:text-xl">
					A real-time multiplayer gaming platform on Stacks. Compete
					in skill-based games, wager STX or tokens, and earn Wars
					Points.
				</p>
				<div className="flex gap-3">
					<Link
						href="/docs"
						className="bg-fd-primary text-fd-primary-foreground hover:bg-fd-primary/90 inline-flex items-center rounded-lg px-6 py-2.5 text-sm font-medium transition-colors"
					>
						Read the Docs
					</Link>
					<Link
						href="/docs/dev/local-setup"
						className="bg-fd-secondary text-fd-secondary-foreground hover:bg-fd-secondary/80 inline-flex items-center rounded-lg px-6 py-2.5 text-sm font-medium transition-colors"
					>
						Get Started
					</Link>
				</div>
			</section>

			{/* Features */}
			<section className="border-fd-border bg-fd-card/50 border-y px-6 py-16">
				<div className="mx-auto grid max-w-5xl gap-8 sm:grid-cols-2 lg:grid-cols-4">
					{features.map((feature) => (
						<div key={feature.title} className="space-y-3">
							<div className="bg-fd-primary/10 text-fd-primary inline-flex rounded-lg p-2.5">
								<feature.icon className="h-5 w-5" />
							</div>
							<h3 className="font-semibold">{feature.title}</h3>
							<p className="text-fd-muted-foreground text-sm leading-relaxed">
								{feature.description}
							</p>
						</div>
					))}
				</div>
			</section>

			{/* Quick Links */}
			<section className="px-6 py-16">
				<div className="mx-auto max-w-5xl">
					<h2 className="mb-8 text-center text-2xl font-bold">
						Quick Links
					</h2>
					<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
						{quickLinks.map((link) => (
							<Link
								key={link.href}
								href={link.href}
								className="border-fd-border hover:bg-fd-accent group rounded-xl border p-5 transition-colors"
							>
								<div className="text-fd-muted-foreground group-hover:text-fd-primary mb-3 transition-colors">
									<link.icon className="h-5 w-5" />
								</div>
								<h3 className="mb-1 font-semibold">
									{link.label}
								</h3>
								<p className="text-fd-muted-foreground text-sm">
									{link.description}
								</p>
							</Link>
						))}
					</div>
				</div>
			</section>

			{/* Footer CTA */}
			<section className="border-fd-border border-t px-6 py-16 text-center">
				<h2 className="mb-3 text-2xl font-bold">Ready to build?</h2>
				<p className="text-fd-muted-foreground mx-auto mb-6 max-w-lg">
					Create your own game engine with the plugin system, or dive
					into the platform docs.
				</p>
				<Link
					href="/docs/dev/building-a-game"
					className="bg-fd-primary text-fd-primary-foreground hover:bg-fd-primary/90 inline-flex items-center rounded-lg px-6 py-2.5 text-sm font-medium transition-colors"
				>
					Build a Game →
				</Link>
			</section>
		</div>
	);
}
