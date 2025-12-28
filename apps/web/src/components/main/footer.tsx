import { Button } from "../ui/button";

export default function Footer() {
	return (
		<footer className="w-full min-h-[60vh] flex flex-col items-center justify-center relative mt-12 md:mt-48">
			<div className="flex flex-col items-center justify-center gap-8 z-10">
				<div className="flex flex-col items-center gap-4">
					<h2
						className="text-3xl md:text-5xl font-bold text-center"
						style={{ color: "var(--color-foreground)" }}
					>
						Every stack counts. Every move
						<br />
						matters. Are you ready?
					</h2>
					<p
						className="text-lg md:text-2xl text-center mt-2"
						style={{ color: "var(--color-foreground)" }}
					>
						Join the Community
					</p>
				</div>
				<div className="flex flex-col md:flex-row gap-6 mt-2">
					<Button
						variant="outline"
						className="px-8 py-4 rounded-full text-base md:text-lg min-w-[200px]"
						asChild
					>
						<a
							href="https://twitter.com"
							target="_blank"
							rel="noopener noreferrer"
						>
							X (Formerly Twitter)
						</a>
					</Button>
					<Button
						variant="outline"
						className="px-8 py-4 rounded-full text-base md:text-lg min-w-[200px]"
						asChild
					>
						<a
							href="https://telegram.org"
							target="_blank"
							rel="noopener noreferrer"
						>
							Telegram
						</a>
					</Button>
				</div>
			</div>

			<div className="w-full flex justify-center">
				<span
					aria-hidden
					className="select-none pointer-events-none font-extrabold text-[15vw] opacity-5 w-full text-center hidden md:block"
					style={{
						color: "var(--color-foreground)",
						letterSpacing: "-0.05em",
					}}
				>
					STACKSWARS
				</span>
			</div>
		</footer>
	);
}
