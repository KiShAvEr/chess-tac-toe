import * as jspb from 'google-protobuf'



export class TakeBackResponse extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TakeBackResponse.AsObject;
  static toObject(includeInstance: boolean, msg: TakeBackResponse): TakeBackResponse.AsObject;
  static serializeBinaryToWriter(message: TakeBackResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TakeBackResponse;
  static deserializeBinaryFromReader(message: TakeBackResponse, reader: jspb.BinaryReader): TakeBackResponse;
}

export namespace TakeBackResponse {
  export type AsObject = {
  }
}

export class JoinRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): JoinRequest.AsObject;
  static toObject(includeInstance: boolean, msg: JoinRequest): JoinRequest.AsObject;
  static serializeBinaryToWriter(message: JoinRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): JoinRequest;
  static deserializeBinaryFromReader(message: JoinRequest, reader: jspb.BinaryReader): JoinRequest;
}

export namespace JoinRequest {
  export type AsObject = {
  }
}

export class TakeBackRequest extends jspb.Message {
  getUuid(): string;
  setUuid(value: string): TakeBackRequest;

  getIsresponse(): boolean;
  setIsresponse(value: boolean): TakeBackRequest;

  getAccepted(): boolean;
  setAccepted(value: boolean): TakeBackRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TakeBackRequest.AsObject;
  static toObject(includeInstance: boolean, msg: TakeBackRequest): TakeBackRequest.AsObject;
  static serializeBinaryToWriter(message: TakeBackRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TakeBackRequest;
  static deserializeBinaryFromReader(message: TakeBackRequest, reader: jspb.BinaryReader): TakeBackRequest;
}

export namespace TakeBackRequest {
  export type AsObject = {
    uuid: string,
    isresponse: boolean,
    accepted: boolean,
  }
}

export class JoinResponse extends jspb.Message {
  getStatus(): JoinResponse.GameStatus;
  setStatus(value: JoinResponse.GameStatus): JoinResponse;

  getUuid(): string;
  setUuid(value: string): JoinResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): JoinResponse.AsObject;
  static toObject(includeInstance: boolean, msg: JoinResponse): JoinResponse.AsObject;
  static serializeBinaryToWriter(message: JoinResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): JoinResponse;
  static deserializeBinaryFromReader(message: JoinResponse, reader: jspb.BinaryReader): JoinResponse;
}

export namespace JoinResponse {
  export type AsObject = {
    status: JoinResponse.GameStatus,
    uuid: string,
  }

  export enum GameStatus { 
    READY = 0,
    NOT_READY = 1,
  }
}

export class SubscribeBoardRequest extends jspb.Message {
  getUuid(): string;
  setUuid(value: string): SubscribeBoardRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SubscribeBoardRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SubscribeBoardRequest): SubscribeBoardRequest.AsObject;
  static serializeBinaryToWriter(message: SubscribeBoardRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SubscribeBoardRequest;
  static deserializeBinaryFromReader(message: SubscribeBoardRequest, reader: jspb.BinaryReader): SubscribeBoardRequest;
}

export namespace SubscribeBoardRequest {
  export type AsObject = {
    uuid: string,
  }
}

export class SubscribeBoardResponse extends jspb.Message {
  getColor(): Color;
  setColor(value: Color): SubscribeBoardResponse;

  getGame(): TicTacToe | undefined;
  setGame(value?: TicTacToe): SubscribeBoardResponse;
  hasGame(): boolean;
  clearGame(): SubscribeBoardResponse;

  getRequest(): MidGameRequest | undefined;
  setRequest(value?: MidGameRequest): SubscribeBoardResponse;
  hasRequest(): boolean;
  clearRequest(): SubscribeBoardResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SubscribeBoardResponse.AsObject;
  static toObject(includeInstance: boolean, msg: SubscribeBoardResponse): SubscribeBoardResponse.AsObject;
  static serializeBinaryToWriter(message: SubscribeBoardResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SubscribeBoardResponse;
  static deserializeBinaryFromReader(message: SubscribeBoardResponse, reader: jspb.BinaryReader): SubscribeBoardResponse;
}

export namespace SubscribeBoardResponse {
  export type AsObject = {
    color: Color,
    game?: TicTacToe.AsObject,
    request?: MidGameRequest.AsObject,
  }
}

export class MidGameRequest extends jspb.Message {
  getTakeback(): Color;
  setTakeback(value: Color): MidGameRequest;

  getDraw(): Color;
  setDraw(value: Color): MidGameRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): MidGameRequest.AsObject;
  static toObject(includeInstance: boolean, msg: MidGameRequest): MidGameRequest.AsObject;
  static serializeBinaryToWriter(message: MidGameRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): MidGameRequest;
  static deserializeBinaryFromReader(message: MidGameRequest, reader: jspb.BinaryReader): MidGameRequest;
}

export namespace MidGameRequest {
  export type AsObject = {
    takeback: Color,
    draw: Color,
  }
}

export class MovePieceRequest extends jspb.Message {
  getBoard(): number;
  setBoard(value: number): MovePieceRequest;

  getAlg(): string;
  setAlg(value: string): MovePieceRequest;

  getUuid(): string;
  setUuid(value: string): MovePieceRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): MovePieceRequest.AsObject;
  static toObject(includeInstance: boolean, msg: MovePieceRequest): MovePieceRequest.AsObject;
  static serializeBinaryToWriter(message: MovePieceRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): MovePieceRequest;
  static deserializeBinaryFromReader(message: MovePieceRequest, reader: jspb.BinaryReader): MovePieceRequest;
}

export namespace MovePieceRequest {
  export type AsObject = {
    board: number,
    alg: string,
    uuid: string,
  }
}

export class Chess extends jspb.Message {
  getWinner(): Color;
  setWinner(value: Color): Chess;

  getFen(): string;
  setFen(value: string): Chess;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Chess.AsObject;
  static toObject(includeInstance: boolean, msg: Chess): Chess.AsObject;
  static serializeBinaryToWriter(message: Chess, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Chess;
  static deserializeBinaryFromReader(message: Chess, reader: jspb.BinaryReader): Chess;
}

export namespace Chess {
  export type AsObject = {
    winner: Color,
    fen: string,
  }
}

export class TicTacToe extends jspb.Message {
  getChessesList(): Array<Chess>;
  setChessesList(value: Array<Chess>): TicTacToe;
  clearChessesList(): TicTacToe;
  addChesses(value?: Chess, index?: number): Chess;

  getNext(): Color;
  setNext(value: Color): TicTacToe;

  getLastmove(): string;
  setLastmove(value: string): TicTacToe;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TicTacToe.AsObject;
  static toObject(includeInstance: boolean, msg: TicTacToe): TicTacToe.AsObject;
  static serializeBinaryToWriter(message: TicTacToe, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TicTacToe;
  static deserializeBinaryFromReader(message: TicTacToe, reader: jspb.BinaryReader): TicTacToe;
}

export namespace TicTacToe {
  export type AsObject = {
    chessesList: Array<Chess.AsObject>,
    next: Color,
    lastmove: string,
  }
}

export class MovePieceResponse extends jspb.Message {
  getSuccessful(): MoveResult;
  setSuccessful(value: MoveResult): MovePieceResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): MovePieceResponse.AsObject;
  static toObject(includeInstance: boolean, msg: MovePieceResponse): MovePieceResponse.AsObject;
  static serializeBinaryToWriter(message: MovePieceResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): MovePieceResponse;
  static deserializeBinaryFromReader(message: MovePieceResponse, reader: jspb.BinaryReader): MovePieceResponse;
}

export namespace MovePieceResponse {
  export type AsObject = {
    successful: MoveResult,
  }
}

export enum MoveResult { 
  RESULT_ERROR = 0,
  RESULT_ILLEGAL = 1,
  RESULT_SUCCESSFUL = 2,
}
export enum Color { 
  WHITE = 0,
  BLACK = 1,
  NONE = 2,
}
