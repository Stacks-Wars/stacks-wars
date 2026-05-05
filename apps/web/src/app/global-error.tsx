"use client";

import { useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Home, RefreshCw, ArrowLeft } from "lucide-react";
import Link from "next/link";

export default function GlobalError({
	error,
	reset,
}: {
	error: Error & { digest?: string };
	reset: () => void;
}) {
	useEffect(() => {
		console.error(error);
	}, [error]);

	return (
		<html>
			<body>
				<div className="bg-app-spotlight text-foreground flex min-h-screen flex-col items-center justify-center bg-fixed px-4">
					<div className="mx-auto max-w-md space-y-6 text-center">
						<div className="space-y-2">
							<h1 className="text-destructive text-6xl font-bold">
								Oops!
							</h1>
							<h2 className="text-2xl font-semibold">
								Something went wrong
							</h2>
							<p className="text-muted-foreground">
								We encountered an unexpected error. This might
								be a temporary issue.
							</p>
						</div>

						{process.env.NODE_ENV === "development" && (
							<div className="bg-muted/50 rounded-lg p-4 text-left">
								<h3 className="mb-2 font-semibold">
									Error Details:
								</h3>
								<p className="font-mono text-sm break-all">
									{error.message}
								</p>
								{error.digest && (
									<p className="text-muted-foreground mt-1 text-xs">
										Digest: {error.digest}
									</p>
								)}
							</div>
						)}

						<div className="flex flex-col justify-center gap-3 sm:flex-row">
							<Button
								onClick={reset}
								className="flex items-center gap-2"
								variant="default"
							>
								<RefreshCw className="size-4" />
								Try Again
							</Button>

							<Button
								onClick={() => window.history.back()}
								className="flex items-center gap-2"
								variant="outline"
							>
								<ArrowLeft className="size-4" />
								Go Back
							</Button>

							<Button asChild variant="outline">
								<Link
									href="/"
									className="flex items-center gap-2"
								>
									<Home className="size-4" />
									Go Home
								</Link>
							</Button>
						</div>

						<p className="text-muted-foreground text-sm">
							If this problem persists, please contact our support
							team.
						</p>
					</div>
				</div>
			</body>
		</html>
	);
}
