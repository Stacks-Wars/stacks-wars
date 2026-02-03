import Image from "next/image";

export default function WinStx() {
	return (
		<div className="w-full md:col-span-2 h-[326.5px] md:h-auto md:!py-20 md:!px-20 lg:!py-28 md:relative md:flex md:items-center flex-row-reverse relative shadow-[0px_0px_34.07px_-18.09px_rgba(255,_255,_255,_0.25)_inset] rounded-3xl [background:linear-gradient(150.88deg,_#2c61b8,_#000_65.11%)] md:[background:linear-gradient(98.77deg,_#2c61b8,_#000_36.25%,_#1d2e4e_62.38%,_#2c61b8_96.98%)] border-darkslategray border-solid border-[0.5px] box-border overflow-hidden">
			<div className="mt-4 mx-3 rounded-[7.82px] md:w-[80%] bg-gray border-darkslategray border-solid border-[0.5px] box-border flex items-center justify-center py-4 px-4 md:!py-10 shrink-0 md:absolute right-0">
				<b className="max-w-[240.5px] md:max-w-[400px] relative leading-[23.46px] inline-block shrink-0 text-center md:text-xl lg:text-2xl text-white">
					<span>
						Win STX and rise in the ranks. The top warriors take home the prize.
					</span>
				</b>
			</div>
			<Image
				className="absolute bottom-0 -left-10 object-cover shrink-0"
				src={"/images/trophy.png"}
				width={369.1}
				height={364.1}
				sizes="100vw"
				alt=""
			/>
		</div>
	);
}
