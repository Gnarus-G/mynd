type Ok<T> = {
  tag: "ok";
  isOk: true;
  data: T;
};

type Err = {
  tag: "err";
  isOk: false;
  error: string;
};

export async function result<T>(p: Promise<T>): Promise<Ok<T> | Err> {
  try {
    const data = await p;
    return {
      tag: "ok",
      isOk: true,
      data,
    };
  } catch (error) {
    return {
      tag: "err",
      isOk: false,
      error: `${error}`,
    };
  }
}

type ResultHandlers<T> = {
  success: (data: T) => void;
  error: (err: string) => void;
};

export function erroneous<T>(
  p: Promise<T>
): (handlers: ResultHandlers<T>) => Promise<void> {
  return async (handlers) => {
    let res = await result(p);
    if (res.tag === "ok") {
      handlers.success(res.data);
    } else {
      handlers.error(res.error);
    }
  };
}
