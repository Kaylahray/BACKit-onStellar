import {
  Controller,
  Post,
  Delete,
  Get,
  Body,
  Param,
  Query,
  HttpCode,
  HttpStatus,
  DefaultValuePipe,
  ParseIntPipe,
  UsePipes,
  ValidationPipe,
} from '@nestjs/common';
import { ApiTags, ApiOperation, ApiParam, ApiResponse } from '@nestjs/swagger';
import { BookmarksService } from './bookmarks.service';
import { CreateBookmarkDto } from './dto/create-bookmark.dto';

@ApiTags('bookmarks')
@Controller('users')
export class BookmarksController {
  constructor(private readonly bookmarksService: BookmarksService) {}

  @Post(':address/bookmarks')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Bookmark a market/call for a user' })
  @ApiParam({ name: 'address', description: 'Wallet address of the user' })
  @ApiResponse({ status: 201, description: 'Bookmark created.' })
  @ApiResponse({ status: 404, description: 'Call not found.' })
  @ApiResponse({ status: 409, description: 'Market already bookmarked.' })
  @UsePipes(new ValidationPipe({ whitelist: true }))
  async addBookmark(
    @Param('address') address: string,
    @Body() dto: CreateBookmarkDto,
  ) {
    return this.bookmarksService.addBookmark(address, dto.callId);
  }

  @Delete(':address/bookmarks/:callId')
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({ summary: 'Remove a bookmark' })
  @ApiParam({ name: 'address', description: 'Wallet address of the user' })
  @ApiParam({ name: 'callId', description: 'Id of the bookmarked call' })
  @ApiResponse({ status: 204, description: 'Bookmark removed.' })
  @ApiResponse({ status: 404, description: 'Bookmark not found.' })
  async removeBookmark(
    @Param('address') address: string,
    @Param('callId') callId: string,
  ) {
    await this.bookmarksService.removeBookmark(address, callId);
  }

  @Get(':address/bookmarks')
  @ApiOperation({
    summary: "Paginated list of a user's bookmarked calls (full call data)",
  })
  @ApiParam({ name: 'address', description: 'Wallet address of the user' })
  @ApiResponse({ status: 200, description: 'Bookmarks retrieved.' })
  async getBookmarks(
    @Param('address') address: string,
    @Query('page', new DefaultValuePipe(1), ParseIntPipe) page: number,
    @Query('limit', new DefaultValuePipe(20), ParseIntPipe) limit: number,
  ) {
    return this.bookmarksService.getBookmarks(address, page, limit);
  }
}
