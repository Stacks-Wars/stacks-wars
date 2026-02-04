import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";

export default function GamesShowcase() {
	return (
		<div className="flex flex-col gap-6 mt-16">
			<div className="self-center flex items-center gap-2 md:gap-3 flex-wrap justify-center">
				<Button className="rounded-full">Hot Games</Button>
				<Button className="bg-popover rounded-full">Hot Games</Button>
				<Button className="bg-popover rounded-full">Hot Games</Button>
				<Button className="bg-popover rounded-full">Hot Games</Button>
			</div>
			<div className="relative -mx-4">
				<div className="overflow-x-auto pb-4 px-4 scrollbar-hide">
					<div className="flex gap-4">
						{Array.from({ length: 8 }).map((_, i) => (
							<Skeleton
								key={i}
								className="w-[240px] h-[186px] lg:w-[445px] lg:h-[334px] rounded-2xl flex-shrink-0"
							/>
						))}
					</div>
				</div>
			</div>
		</div>
	);
}
