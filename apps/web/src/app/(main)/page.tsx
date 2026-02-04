import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Gamepad2, Users, Trophy, Zap, Shield } from "lucide-react";

export default async function HomePage() {
	return (
		<div className="flex flex-col">
			{/* Hero Section */}
			<section className="relative overflow-hidden">
				<div className="relative container mx-auto px-4 py-16 sm:py-24 lg:py-32">
					<div className="mx-auto max-w-4xl text-center">
						<Badge
							variant="secondary"
							className="mb-6 px-4 py-2 text-sm"
						>
							🎮 Compete. Win. Earn STX.
						</Badge>
						<h1 className="mb-6 text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl">
							Where Gaming Meets{" "}
							<span className="from-primary to-primary/60 bg-linear-to-r bg-clip-text text-transparent">
								Blockchain
							</span>
						</h1>
						<p className="text-muted-foreground mx-auto mb-8 max-w-2xl text-lg sm:text-xl">
							Join the ultimate multiplayer gaming platform on
							Stacks. Challenge players worldwide, compete in
							skill-based games, and win real STX rewards.
						</p>
						<div className="flex flex-col items-center justify-center gap-4 sm:flex-row">
							<Button
								asChild
								size="lg"
								className="w-full px-8 sm:w-auto"
							>
								<Link href="/games">
									<Gamepad2 className="mr-2 h-5 w-5" />
									Browse Games
								</Link>
							</Button>
							<Button
								asChild
								variant="outline"
								size="lg"
								className="w-full px-8 sm:w-auto"
							>
								<Link href="/lobby">
									<Users className="mr-2 h-5 w-5" />
									Join a Lobby
								</Link>
							</Button>
						</div>
					</div>
				</div>
			</section>

			{/* Features Section */}
			<section className="border-y py-16 sm:py-20">
				<div className="container mx-auto px-4">
					<div className="grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
						<FeatureCard
							icon={Trophy}
							title="Win STX Rewards"
							description="Compete in skill-based games and win real cryptocurrency prizes"
						/>
						<FeatureCard
							icon={Shield}
							title="Secure & Fair"
							description="All games are verified on-chain for transparent and fair gameplay"
						/>
						<FeatureCard
							icon={Zap}
							title="Instant Matches"
							description="Find opponents quickly and start playing within seconds"
						/>
						<FeatureCard
							icon={Users}
							title="Growing Community"
							description="Join thousands of players competing across multiple games"
						/>
					</div>
				</div>
			</section>

			{/* CTA Section */}
			<section className="py-16 sm:py-24">
				<div className="container mx-auto px-4">
					<div className="bg-gradient-primary relative overflow-hidden rounded-3xl border p-8 sm:p-12 lg:p-16">
						<div className="relative z-10 mx-auto max-w-2xl text-center">
							<h2 className="mb-4 text-2xl font-bold sm:text-3xl lg:text-4xl">
								Ready to Start Winning?
							</h2>
							<p className="text-muted-foreground mb-8 text-base sm:text-lg">
								Create your account, join a game, and start
								earning STX rewards today. The battlefield
								awaits!
							</p>
							<div className="flex flex-col items-center justify-center gap-4 sm:flex-row">
								<Button
									asChild
									size="lg"
									className="w-full px-8 sm:w-auto"
								>
									<Link href="/signup">Get Started Free</Link>
								</Button>
								<Button
									asChild
									variant="outline"
									size="lg"
									className="w-full px-8 sm:w-auto"
								>
									<Link href="/leaderboard">
										View Leaderboard
									</Link>
								</Button>
							</div>
						</div>
						<div className="absolute -top-24 -right-24 h-64 w-64 rounded-full bg-purple-500/10 blur-3xl" />
						<div className="absolute -bottom-24 -left-24 h-64 w-64 rounded-full bg-blue-500/10 blur-3xl" />
					</div>
				</div>
			</section>
		</div>
	);
}

function FeatureCard({
	icon: Icon,
	title,
	description,
}: {
	icon: React.ElementType;
	title: string;
	description: string;
}) {
	return (
		<div className="group text-center">
			<div className="bg-primary/10 group-hover:bg-primary/20 mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl transition-colors">
				<Icon className="text-primary h-7 w-7" />
			</div>
			<h3 className="mb-2 text-lg font-semibold">{title}</h3>
			<p className="text-muted-foreground text-sm">{description}</p>
		</div>
	);
}
