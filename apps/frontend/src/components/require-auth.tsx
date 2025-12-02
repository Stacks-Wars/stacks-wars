import ConnectWallet from "./home/connect-wallet";

export default function RequireAuth() {
	return (
		<div className="min-h-screen flex flex-col items-center justify-center px-4">
			<div className="text-center space-y-6 max-w-md flex flex-col items-center">
				<div className="space-y-2">
					<h1 className="text-2xl font-bold text-foreground">
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
