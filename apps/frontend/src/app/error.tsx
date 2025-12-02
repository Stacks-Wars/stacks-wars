"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { RefreshCw, Home, Bug } from "lucide-react";

export default function GlobalError({
	error,
}: {
	error: Error;
	reset: () => void;
}) {
	const router = useRouter();

	useEffect(() => {
		console.error("🔥 Unhandled error:", error);
	}, [error]);

	const handleRefresh = () => {
		window.location.reload();
	};

	const handleGoHome = () => {
		router.push("/");
	};

	return (
		<div className="min-h-screen flex items-center justify-center bg-gradient-to-b from-background to-primary/30">
			<div className="text-center space-y-8 px-4">
				{/* Error Code */}
				<div className="space-y-2">
					<h1 className="text-8xl font-bold text-destructive/20">
						500
					</h1>
					<h2 className="text-2xl sm:text-3xl font-bold text-foreground">
						Something Went Wrong
					</h2>
					<p className="text-muted-foreground max-w-md mx-auto">
						An unexpected error occurred. We&apos;re working on it,
						but you can try refreshing or head back home.
					</p>
				</div>

				<div className="flex flex-col sm:flex-row gap-3 justify-center items-center">
					<Button
						onClick={handleRefresh}
						variant="outline"
						className="w-full sm:w-auto"
					>
						<RefreshCw className="mr-2 h-4 w-4" />
						Retry
					</Button>

					<Button onClick={handleGoHome} className="w-full sm:w-auto">
						<Home className="mr-2 h-4 w-4" />
						Go Home
					</Button>
				</div>

				<div className="pt-4 border-t border-destructive/30">
					<p className="text-sm text-muted-foreground">
						If the problem persists, please report it or check back
						later. <Bug className="inline h-4 w-4 ml-1" />
					</p>
				</div>
			</div>
		</div>
	);
}
