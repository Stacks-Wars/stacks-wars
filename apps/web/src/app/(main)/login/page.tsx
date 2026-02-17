import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import HandleConnect from "@/components/main/handle-connect";
import { Suspense } from "react";

function LoginContent() {
	return (
		<Card>
			<CardContent className="sm:max-w-md">
				<CardHeader>
					<CardTitle>Connect Wallet</CardTitle>
					<CardDescription>
						Connect your Stacks wallet to join games and compete
						with other players.
					</CardDescription>
				</CardHeader>
				<div className="flex flex-col gap-4">
					<HandleConnect />
					<p className="text-muted-foreground text-center text-xs">
						By connecting your wallet, you agree to our Terms of
						Service and Privacy Policy.
					</p>
				</div>
			</CardContent>
		</Card>
	);
}

export default function LoginPage() {
	return (
		<div className="flex min-h-screen items-center justify-center p-4">
			<Suspense fallback={<LoginContent />}>
				<LoginContent />
			</Suspense>
		</div>
	);
}
