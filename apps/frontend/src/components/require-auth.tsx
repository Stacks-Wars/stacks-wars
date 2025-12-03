import ConnectWallet from "./home/connect-wallet";

export default function RequireAuth() {
	return (
		<div className="flex min-h-screen flex-col items-center justify-center px-4">
			<div className="flex max-w-md flex-col items-center space-y-6 text-center">
				<div className="space-y-2">
					<h1 className="text-foreground text-2xl font-bold">
						Wallet Connection Required
					</h1>
					<p className="text-muted-foreground">
						You need to connect your wallet to access this page.
					</p>
				</div>
				<ConnectWallet />
			</div>
		</div>
	);
}
