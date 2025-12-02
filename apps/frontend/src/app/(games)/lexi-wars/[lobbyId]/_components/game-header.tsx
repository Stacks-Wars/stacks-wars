import { Brain } from "lucide-react";

export default function GameHeader() {
	return (
		<div className="w-full border rounded-xl shadow-sm bg-primary/30">
			<div className="flex items-center justify-between gap-2 p-3 sm:p-4">
				<div className="flex items-center gap-2 sm:gap-3">
					<div className="size-12 rounded-full bg-primary/10 flex items-center justify-center">
						<Brain className="size-6 text-primary" />
					</div>
					<div>
						<p className="font-medium text-xl">Lexi War</p>
						<p className="text-muted-foreground text-sm">
							Word Battle Royale
						</p>
					</div>
				</div>
				{/*<div className="flex items-center gap-2 sm:gap-3">
					<Badge
						variant="outline"
						className="text-lg px-3 sm:px-4 py-1 sm:py-2"
					>
						<Target className="mr-1" />
						{score}
					</Badge>
					<Badge
						variant="outline"
						className="text-lg px-3 sm:px-4 py-1 sm:py-2 bg-primary/10"
					>
						<Trophy className="mr-1" />
						{highScore}
					</Badge>
				</div>*/}
			</div>
		</div>
	);
}
