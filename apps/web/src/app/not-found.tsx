"use client";
import { Button } from "@/components/ui/button";
import { Home, ArrowLeft, Search } from "lucide-react";
import Link from "next/link";

export default function NotFound() {
	return (
		<div className="flex min-h-screen flex-col items-center justify-center px-4">
			<div className="mx-auto max-w-md space-y-6 text-center">
				<div className="space-y-2">
					<h1 className="text-muted-foreground text-8xl font-bold">
						404
					</h1>
					<h2 className="text-2xl font-semibold">Page Not Found</h2>
					<p className="text-muted-foreground">
						The page you're looking for doesn't exist or has been
						moved.
					</p>
				</div>

				<div className="flex flex-col justify-center gap-3 sm:flex-row">
					<Button
						onClick={() => window.history.back()}
						className="flex items-center gap-2"
						variant="default"
					>
						<ArrowLeft className="size-4" />
						Go Back
					</Button>

					<Button asChild variant="outline">
						<Link href="/" className="flex items-center gap-2">
							<Home className="size-4" />
							Go Home
						</Link>
					</Button>

					<Button asChild variant="outline">
						<Link href="/games" className="flex items-center gap-2">
							<Search className="size-4" />
							Browse Games
						</Link>
					</Button>
				</div>

				<p className="text-muted-foreground text-sm">
					Looking for something specific? Try browsing our game
					collection.
				</p>
			</div>
		</div>
	);
}
