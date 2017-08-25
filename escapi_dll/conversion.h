#pragma once

typedef void(*IMAGE_TRANSFORM_FN)(
	BYTE*       aDest,
	LONG        aDestStride,
	const BYTE* aSrc,
	LONG        aSrcStride,
	DWORD       aWidthInPixels,
	DWORD       aHeightInPixels,
	DWORD		bufferLength
	);

struct ConversionFunction
{
	GUID               mSubtype;
	IMAGE_TRANSFORM_FN mXForm;
};

void TransformImage_RGB24(
	BYTE*       aDest,
	LONG        aDestStride,
	const BYTE* aSrc,
	LONG        aSrcStride,
	DWORD       aWidthInPixels,
	DWORD       aHeightInPixels,
	DWORD		bufferLength
	);

void TransformImage_RGB32(
	BYTE*       aDest,
	LONG        aDestStride,
	const BYTE* aSrc,
	LONG        aSrcStride,
	DWORD       aWidthInPixels,
	DWORD       aHeightInPixels,
	DWORD		bufferLength
	);

void TransformImage_YUY2(
	BYTE*       aDest,
	LONG        aDestStride,
	const BYTE* aSrc,
	LONG        aSrcStride,
	DWORD       aWidthInPixels,
	DWORD       aHeightInPixels,
	DWORD		bufferLength
	);

void TransformImage_NV12(
	BYTE*		aDst,
	LONG		aDestStride,
	const BYTE* aSrc,
	LONG		aSrcStride,
	DWORD		aWidthInPixels,
	DWORD		aHeightInPixels,
	DWORD		bufferLength
	);

void TransformImage_MJPG(
	BYTE*       aDest,
	LONG        aDestStride,
	const BYTE* aSrc,
	LONG        aSrcStride,
	DWORD       aWidthInPixels,
	DWORD       aHeightInPixels,
	DWORD		bufferLength
);

void DummyTransform(
	BYTE*       aDest,
	LONG        aDestStride,
	const BYTE* aSrc,
	LONG        aSrcStride,
	DWORD       aWidthInPixels,
	DWORD       aHeightInPixels,
	DWORD		bufferLength
);


extern ConversionFunction gFormatConversions[];
extern const DWORD gConversionFormats;
