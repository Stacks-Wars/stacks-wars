"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { useAuthStore } from "@/lib/stores/auth";

export default function HomePage() {
	const { isAuthenticated, user } = useAuthStore();

	return (
		<div className="container mx-auto px-4 py-8">
			{/*HomePage*/}
			{/*<div className="flex flex-col gap-8">
				<div className="flex items-center justify-between">
					<h1 className="text-4xl font-bold">Stacks Wars</h1>
					<div className="flex items-center gap-4">
						{isAuthenticated ? (
							<>
								<Link href={`/u/${user?.id}`}>
									<Button variant="outline">Profile</Button>
								</Link>
								<Link href="/games">
									<Button>Browse Games</Button>
								</Link>
							</>
						) : (
							<Link href="/login">
								<Button size="lg">Connect Wallet</Button>
							</Link>
						)}
					</div>
				</div>

				{isAuthenticated ? (
					<div className="grid gap-4">
						<h2 className="text-2xl font-semibold">
							Welcome,{" "}
							{user?.displayName ||
								user?.username ||
								user?.walletAddress ||
								"Player"}
							!
						</h2>
						<p className="text-muted-foreground">
							Get started by browsing games or managing your
							profile.
						</p>
					</div>
				) : (
					<div className="grid gap-4 py-12 text-center">
						<h2 className="text-3xl font-semibold">
							Welcome to Stacks Wars
						</h2>
						<p className="text-lg text-muted-foreground">
							Connect your wallet to start playing games and
							competing with others
						</p>
						<div className="mt-4">
							<Link href="/login">
								<Button size="lg">Connect Wallet</Button>
							</Link>
						</div>
					</div>
				)}
			</div>*/}
		</div>
	);
}
