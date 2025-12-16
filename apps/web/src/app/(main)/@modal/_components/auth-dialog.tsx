"use client";

import { SiGoogle } from "@icons-pack/react-simple-icons";
import {
	connect,
	disconnect,
	getLocalStorage,
	isConnected as isWalletConnected,
	request,
} from "@stacks/connect";
import { DOMAIN_NAME, siteConfig } from "@stacks-wars/shared";
import { CheckCircle2, Loader2, Wallet } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import type React from "react";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { authClient } from "@/lib/auth-client";

type AuthMode = "login" | "signup";
type AuthType = "wallet" | "google";

interface AuthDialogProps {
	trigger?: React.ReactNode;
	open?: boolean;
	onOpenChange?: (open: boolean) => void;
	mode?: AuthMode;
}

export function AuthDialog({
	trigger,
	open,
	onOpenChange,
	mode = "login",
}: AuthDialogProps) {
	const [isConnecting, setIsConnecting] = useState<AuthType | null>(null);
	const [isConnected, setIsConnected] = useState<AuthType | null>(null);
	const router = useRouter();

	const isSignup = mode === "signup";
	const title = isSignup ? "Create your account" : "Sign in to your account";
	const description = isSignup
		? "Choose your preferred method to create an account"
		: "Choose your preferred authentication method to continue";
	const walletText = isSignup ? "Sign up with Wallet" : "Connect Wallet";
	const googleText = isSignup ? "Sign up with Google" : "Continue with Google";

	const handleWalletConnect = async () => {
		setIsConnecting("wallet");
		try {
			if (isWalletConnected()) {
				disconnect();
			}

			await connect({ network: "mainnet" });
			if (!isWalletConnected()) {
				toast.error("Failed to connect to wallet");
				return;
			}

			const walletData = getLocalStorage();
			const address = walletData?.addresses?.stx?.[0]?.address;

			if (!address) {
				toast.error("No Stacks address found");
				return;
			}

			const { data: nonceData, error: nonceError } =
				await authClient.siws.nonce({
					walletAddress: address,
				});

			if (nonceError || !nonceData?.nonce) {
				toast.error("Failed to generate authentication nonce");
				return;
			}

			const message = `Sign in to ${siteConfig.title} ${DOMAIN_NAME} ${nonceData.nonce}`;

			const signResponse = await request("stx_signMessage", {
				message: message,
			});

			if (!signResponse?.publicKey || !signResponse?.signature) {
				toast.error("Message signing was cancelled or failed");
				return;
			}

			const { data: verificationData, error: verificationError } =
				await authClient.siws.verify({
					message: message,
					signature: signResponse.signature,
					walletAddress: address,
					publicKey: signResponse.publicKey,
				});

			if (verificationError) {
				toast.error("Authentication verification failed");
				console.error("Verification error:", verificationError);
				return;
			}

			if (verificationData) {
				toast.success("Authenticated");
				router.push("/games");
				setIsConnected("wallet");
				// return verificationData;
			}
		} catch (error) {
			console.error("Stacks authentication error:", error);

			if (error instanceof Error) {
				if (error.message?.includes("network")) {
					toast.error("Network error - please check your connection");
				} else if (error.message?.includes("user")) {
					toast.error("Authentication cancelled by user");
				} else {
					toast.error("Authentication failed - please try again");
				}
			}
		} finally {
			setIsConnecting(null);
		}
	};

	const handleGoogleConnect = async () => {
		setIsConnecting("google");
		// Simulate Google OAuth
		await new Promise((resolve) => setTimeout(resolve, 2000));
		setIsConnecting(null);
		setIsConnected("google");
	};

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			{trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
			<DialogContent className="sm:max-w-md border-border bg-card">
				<DialogHeader className="text-center space-y-3">
					<div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-accent/10 border border-accent/20">
						<div className="h-3 w-3 rounded-full bg-accent" />
					</div>
					<DialogTitle className="text-xl font-semibold text-foreground">
						{title}
					</DialogTitle>
					<DialogDescription className="text-muted-foreground">
						{description}
					</DialogDescription>
				</DialogHeader>

				<div className="mt-6 space-y-3">
					<Button
						variant="secondary"
						size={"lg"}
						className="w-full"
						// className="w-full h-12 justify-start gap-4 bg-secondary hover:bg-secondary/80 text-secondary-foreground border border-border transition-all duration-200"
						onClick={handleWalletConnect}
						disabled={isConnecting !== null}
					>
						{isConnecting === "wallet" ? (
							<Loader2 className="h-5 w-5 animate-spin" />
						) : isConnected === "wallet" ? (
							<CheckCircle2 className="h-5 w-5 text-accent" />
						) : (
							<Wallet className="h-5 w-5" />
						)}
						<span className="flex-1 text-left font-medium">
							{isConnecting === "wallet"
								? "Connecting wallet..."
								: isConnected === "wallet"
									? "Wallet connected"
									: walletText}
						</span>
						{!isConnecting && !isConnected && (
							<span className="text-xs text-muted-foreground">
								Leather, Xverse...
							</span>
						)}
					</Button>

					<div className="relative">
						<div className="absolute inset-0 flex items-center">
							<div className="w-full border-t border-border" />
						</div>
						<div className="relative flex justify-center text-xs">
							<span className="bg-card px-3 text-muted-foreground">or</span>
						</div>
					</div>

					<Button
						variant="secondary"
						size={"lg"}
						className="w-full"
						onClick={handleGoogleConnect}
						disabled={true}
						// disabled={isConnecting !== null}
					>
						{isConnecting === "google" ? (
							<Loader2 className="h-5 w-5 animate-spin" />
						) : isConnected === "google" ? (
							<CheckCircle2 className="h-5 w-5 text-accent" />
						) : (
							<SiGoogle size={14} title="X icon" className="" />
						)}
						<span className="flex-1 text-left font-medium">
							{isConnecting === "google"
								? isSignup
									? "Creating account..."
									: "Signing in..."
								: isConnected === "google"
									? isSignup
										? "Account created"
										: "Signed in with Google"
									: googleText}
						</span>
					</Button>
				</div>

				<p className="mt-6 text-center text-xs text-muted-foreground">
					{isSignup ? (
						<>
							Already have an account?{" "}
							<a href="#" className="text-accent hover:underline">
								Sign in
							</a>
						</>
					) : (
						<>
							Don't have an account?{" "}
							<a href="#" className="text-accent hover:underline">
								Create one
							</a>
						</>
					)}
				</p>
				<p className="text-center text-xs text-muted-foreground">
					By continuing, you agree to our{" "}
					<Link
						href="/terms-of-service"
						className="text-accent hover:underline"
					>
						Terms of Service
					</Link>{" "}
					and{" "}
					<Link href="/privacy-policy" className="text-accent hover:underline">
						Privacy Policy
					</Link>
				</p>
			</DialogContent>
		</Dialog>
	);
}
