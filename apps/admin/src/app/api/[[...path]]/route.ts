import { NextRequest } from 'next/server';
import { handleApiProxy, handleOptionsRequest } from '@/lib/api-proxy';

export async function GET(
  request: NextRequest,
  { params }: { params: { path?: string[] } },
) {
  return handleApiProxy(request, params);
}

export async function POST(
  request: NextRequest,
  { params }: { params: { path?: string[] } },
) {
  return handleApiProxy(request, params);
}

export async function PUT(
  request: NextRequest,
  { params }: { params: { path?: string[] } },
) {
  return handleApiProxy(request, params);
}

export async function PATCH(
  request: NextRequest,
  { params }: { params: { path?: string[] } },
) {
  return handleApiProxy(request, params);
}

export async function DELETE(
  request: NextRequest,
  { params }: { params: { path?: string[] } },
) {
  return handleApiProxy(request, params);
}

export async function OPTIONS() {
  return handleOptionsRequest();
}
