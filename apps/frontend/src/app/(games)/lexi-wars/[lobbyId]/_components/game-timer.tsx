import { Badge } from "@/components/ui/badge";
import { MdOutlineTimer } from "react-icons/md";
import { cn } from "@/lib/utils";
export default function GameTimer({ timeLeft }: { timeLeft: number }) {
	return (
		<div
			className={`${cn(
				timeLeft <= 5
					? "bg-destructive/50 border-destructive/90"
					: "bg-primary/10",
				"flex w-full items-center justify-center rounded-xl border"
			)}`}
		>
			<div className="flex w-full items-center justify-between gap-2 p-3 sm:p-4">
				<p className="text-base font-medium">Time Left</p>
				<Badge
					variant={timeLeft <= 5 ? "destructive" : "default"}
					className="text-foreground px-3 py-1 text-base font-bold sm:px-4 sm:py-2"
				>
					<MdOutlineTimer className="mr-1 size-4" />
					{timeLeft}s
				</Badge>
			</div>
		</div>
	);
}
